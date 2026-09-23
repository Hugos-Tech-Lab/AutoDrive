use std::{ffi::c_void, rc::Rc};

use crate::{
    auto_script::{
        AutoScriptRunProgress, WasmResponse,
        cancellation_token::{self, CancellationToken},
    }, hardware::on_board_led::OnBoardLed, inter_thread::InterThreadListener, utils::heap,
};
use anyhow::{Result, anyhow};
use flume::Sender;
use log::info;
use smart_leds_trait::RGB8;
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
    pub ct: CancellationToken,
}

impl Drop for WasmData {
    fn drop(&mut self) {
        OnBoardLed::set_color(RGB8 { r: 0, g: 0, b: 0 })
    }
}

pub struct WasmThread {
    runtime: Rc<Runtime>,
    installed_module: Option<Rc<Module>>,
    ct: CancellationToken,
}

impl WasmThread {
    pub fn new(ct: CancellationToken) -> Result<Self> {
        let hea = heap();
        dbg!("before alloc {:?}", hea);
        let runtime_pool = vec![0u8; 32 * 1024].into_boxed_slice();
        let linear_pool = vec![0u8; 64 * 1024 + 32].into_boxed_slice();
        dbg!("1111111111111111111111111");
        let hea = heap();
        dbg!("after alloc {:?}", hea);
        let runtime = Runtime::builder()
            .use_memory_pool(runtime_pool, linear_pool)
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

        dbg!("2222222222222222222222222");

        Ok(Self {
            runtime: Rc::new(runtime),
            installed_module: None,
            ct,
        })
    }

    pub fn listen(
        &mut self,
        listener: &InterThreadListener<WasmThreadCommand, Result<()>>,
    ) -> Result<()> {
        info!("WASM runtime started listening for requests.");
        loop {
            match listener.listen() {
                Ok((command, response)) => match command {
                    WasmThreadCommand::Install { data } => {
                        let ret = self.install(data);
                        response.send(ret)?;
                    }
                    WasmThreadCommand::Run { progress } => {
                        let ret = self.run(progress, self.ct.clone());
                        response.send(ret)?;
                    }
                },
                Err(error) => {
                    return Err(anyhow!("error listening in wasm thread: {:?}", error));
                }
            }
        }
    }

    fn install(&mut self, data: Vec<u8>) -> Result<()> {
        self.installed_module = None;

        let module = Module::from_vec(self.runtime.clone(), data, "env")
            .map_err(|e| anyhow::anyhow!("failed to create module: {e:?}"))?;

        let module = self.installed_module.insert(Rc::new(module)).clone();

        self.installed_module = Some(module);
        Ok(())
    }

    pub fn run(
        &mut self,
        progress: Sender<AutoScriptRunProgress>,
        ct: CancellationToken,
    ) -> Result<()> {
        info!("running");
        progress.send(AutoScriptRunProgress::ReceivedRunAction).unwrap();

        let installed_module = self.installed_module.clone().ok_or(anyhow::anyhow!(
            "installed module not found. did you install first before running?"
        ))?;

        progress.send(AutoScriptRunProgress::InstantiatingInstance).unwrap();
        let instance = Rc::new(
            Instance::new(installed_module.clone(), 1024 * 16)
                .map_err(|e| format!("failed to create instance: {}", e)) // TODO these should be more specific types for wasm and no unwrap AND CLEANUP TOO
                .unwrap(),
        );

        let mut wasm_data = Box::new(WasmData {
            progress: progress.clone(),
            ct,
        });

        unsafe {
            wasm_runtime_set_custom_data(
                instance.get_inner_instance(),
                wasm_data.as_mut() as *mut WasmData as *mut c_void,
            )
        }; //  TODO: disable WAMR_BUILD_LIB_PTHREAD

        progress.send(AutoScriptRunProgress::FindingMain).unwrap();

        let main_function = Function::find_export_func(instance.clone(), "main").unwrap();


        progress.send(AutoScriptRunProgress::CallingMain).unwrap();
        let res = main_function.call(&vec![]);
        log::info!("{:?}", res);

        // cleanup after cancellation
        if wasm_data.ct.is_cancelled() {
            log::info!("cancelled");
        }

        log::info!("done");
        Ok(())
    }
}
