use std::{
    ffi::c_void,
    rc::Rc,
};

use crate::{
    auto_script::{
        AutoScriptRunProgress, WasmResponse, cancellation_token,
        exposed_functions::CURRENT_RUN_PROGRESS,
    },
    inter_thread::{InterThreadListener},
};
use anyhow::{Result};
use flume::Sender;
use log::info;
use wamr_rust_sdk::{function::Function, instance::Instance, module::Module, runtime::Runtime};

pub enum WasmThreadCommand {
    Install {
        data: Vec<u8>,
    },
    Run {
        progress: Sender<AutoScriptRunProgress>,
    },
}

pub struct WasmThread {
    runtime: Rc<Runtime>,
    installed_module: Option<Rc<Module>>,
    running_instance: Option<Rc<Instance>>,
    running_instance_main: Option<Function>,
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
            running_instance: None,
            running_instance_main: None,
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
                        let mut current_run_progress = CURRENT_RUN_PROGRESS.lock().unwrap();
                        *current_run_progress = Some(progress.clone());
                        self.run();
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
        self.running_instance_main = None;
        self.running_instance = None;
        self.installed_module = None;

        let module = self
            .installed_module
            .insert(Rc::new(
                Module::from_vec(self.runtime.clone(), data, "env")
                    .map_err(|e| format!("failed to create module: {e:?}"))
                    .unwrap(),
            ))
            .clone();

        self.instantiate(module.clone());
        Ok(())
    }

    fn instantiate(&mut self, module: Rc<Module>) {
        self.running_instance_main = None;
        self.running_instance = None;
        self.installed_module = Some(module.clone());

        let instance = self.running_instance.insert(
            Instance::new(module.clone(), 1024 * 16)
                .map_err(|e| format!("failed to create instance: {}", e)) // TODO these should be more specific types for wasm and no unwrap AND CLEANUP TOO
                .unwrap()
                .into(),
        );

        self.running_instance_main =
            Some(Function::find_export_func(instance.clone(), "main").unwrap());
    }

    pub fn run(&mut self) {
        info!("running");
        if let Some(ref main_function) = self.running_instance_main {
            let res = main_function.call(&vec![]);
            log::info!("{:?}", res);
        } else {
            // TODO: tell the user
        }

        // cleanup after cancellation
        if cancellation_token::is_cancelled() {
            let module = self.installed_module.clone().unwrap();

            log::info!("cancelled");
            self.instantiate(module)
        }
        log::info!("done");
    }
}
