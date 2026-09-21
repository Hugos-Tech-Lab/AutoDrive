use std::{ffi::c_void, rc::Rc};

use crate::{
    auto_script::{AutoScriptRunProgress, WasmResponse, cancellation_token},
    inter_thread::InterThreadListener,
};
use anyhow::Result;
use flume::Sender;
use log::info;
use wamr_rust_sdk::{
    function::Function, instance::Instance, module::Module, runtime::Runtime,
    sys::wasm_runtime_set_custom_data,
};

pub enum WasmThreadCommand {
    Install {
        data: Vec<u8>,
    },
    Run {
        progress: Sender<AutoScriptRunProgress>,
    },
}

pub struct WasmData {
    pub progress: flume::Sender<AutoScriptRunProgress>,
}

pub struct WasmThread {
    runtime: Rc<Runtime>,
    installed_module: Option<Rc<Module>>,
}

impl WasmThread {
    pub fn new() -> Result<Self> {
        let runtime = Runtime::builder()
            .use_system_allocator()
            .register_host_function(
                "delay",
                crate::auto_script::exposed_functions::delay as *mut c_void,
            )
            .register_host_function(
                "print",
                crate::auto_script::exposed_functions::print as *mut c_void,
            )
            .register_host_function(
                "set_onboard_led_color",
                crate::auto_script::exposed_functions::set_onboard_led_color as *mut c_void,
            )
            .build()?;

        Ok(Self {
            runtime: Rc::new(runtime),
            installed_module: None,
        })
    }

    pub fn listen(
        &mut self,
        listener: &InterThreadListener<WasmThreadCommand, WasmResponse>,
    ) -> Result<()> {
        info!("WASM runtime started");
        loop {
            match listener.listen() {
                Ok((command, response)) => match command {
                    WasmThreadCommand::Install { data } => {
                        self.install(data).unwrap();
                        response.send(WasmResponse::SuccessfullyInstalled)?;
                    }
                    WasmThreadCommand::Run { progress } => {
                        self.run(progress);
                    }
                },
                Err(error) => {
                    log::error!("error listening in wasm thread: {:?}", error);
                    break;
                }
            }
        }

        info!("WASM thread stopped");
        Ok(())
    }

    fn install(&mut self, data: Vec<u8>) -> Result<(), WasmResponse> {
        self.installed_module = None;

        let module = self
            .installed_module
            .insert(Rc::new(
                Module::from_vec(self.runtime.clone(), data, "env")
                    .map_err(|e| format!("failed to create module: {e:?}"))
                    .unwrap(),
            ))
            .clone();

        self.installed_module = Some(module);
        Ok(())
    }

    pub fn run(&mut self, progress: Sender<AutoScriptRunProgress>) {
        info!("running");

        let Some(ref installed_module) = self.installed_module else {
            // TODO: give error module should be installed
            return;
        };

        let instance = Rc::new(
            Instance::new(installed_module.clone(), 1024 * 16)
                .map_err(|e| format!("failed to create instance: {}", e)) // TODO these should be more specific types for wasm and no unwrap AND CLEANUP TOO
                .unwrap(),
        );

        let mut wasm_data = Box::new(WasmData { progress });

        unsafe {
            wasm_runtime_set_custom_data(
                instance.get_inner_instance(),
                wasm_data.as_mut() as *mut WasmData as *mut c_void,
            )
        }; //  disable WAMR_BUILD_LIB_PTHREAD

        let main_function = Function::find_export_func(instance.clone(), "main").unwrap();

        let res = main_function.call(&vec![]);
        log::info!("{:?}", res);

        // cleanup after cancellation
        if cancellation_token::is_cancelled() {
            log::info!("cancelled");
        }
        log::info!("done");
    }
}
