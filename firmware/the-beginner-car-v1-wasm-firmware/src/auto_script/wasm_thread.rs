use std::{
    ffi::c_void,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, AtomicUsize, Ordering},
        mpsc::{Receiver, SyncSender},
    },
};

use crate::{
    auto_script::{
        cancellation_token::CancellationToken, exposed_functions::TheBeginnerCarExports,
    },
    the_beginner_car_hardware::TheBeginnerCarHardware,
};
use anyhow::{Result, anyhow};
use esp_idf_sys::xTaskGetTickCount;
use log::{error, info};
use the_beginner_car_wasm_sdk::host::host_functions;
use wamr_rust_sdk::{
    function::Function, instance::Instance, module::Module, runtime::Runtime,
    sys::wasm_runtime_set_custom_data,
};

pub enum WasmThreadCommand {
    Run { data: Vec<u8> },
}

pub struct WasmData {
    pub ct: CancellationToken,
    pub hardware: Arc<TheBeginnerCarHardware>,
    pub last_host_fn_call_ticks_ms: Arc<AtomicU32>,
}

impl Drop for WasmData {
    fn drop(&mut self) {
        // OnBoardLed::set_color(RGB8 { r: 0, g: 0, b: 0 })
    }
}

pub struct WasmThread {
    hardware: Arc<TheBeginnerCarHardware>,
    runtime: Rc<Runtime>,
    on_stopped_running: Box<dyn Fn() + Send + 'static>,
    ct: CancellationToken,
    active_wasm_instance: Arc<AtomicUsize>,
    last_host_fn_call_ticks_ms: Arc<AtomicU32>,
}

impl WasmThread {
    pub fn new(
        hardware: Arc<TheBeginnerCarHardware>,
        active_wasm_instance: Arc<AtomicUsize>,
        on_stopped_running: impl Fn() + Send + 'static,
        last_host_fn_call_ticks_ms: Arc<AtomicU32>,
        ct: CancellationToken,
    ) -> Result<Self> {
        let runtime_pool = vec![0u8; 32 * 1024].into_boxed_slice();
        // 64kb is one page
        // ~1kb for static strings
        // ~1kb for allocator header
        let linear_pool = vec![0u8; 66 * 1024].into_boxed_slice();

        let mut runtime = Runtime::builder().use_memory_pool(runtime_pool, linear_pool);

        for native_sybol in host_functions::<TheBeginnerCarExports>() {
            runtime = runtime.register_host_function(native_sybol.f_name, native_sybol.f_pointer)
        }
        let runtime = runtime.build()?;

        Ok(Self {
            hardware,
            runtime: Rc::new(runtime),
            ct,
            on_stopped_running: Box::new(on_stopped_running),
            last_host_fn_call_ticks_ms,
            active_wasm_instance,
        })
    }

    pub fn listen(
        &mut self,
        receiver: &Receiver<(WasmThreadCommand, SyncSender<Result<()>>)>,
    ) -> Result<()> {
        info!("WASM runtime started listening for requests.");
        loop {
            match receiver.recv() {
                Ok((command, sender)) => match command {
                    WasmThreadCommand::Run { data } => {
                        let ret = self.run(data, sender, self.ct.clone());
                        if let Err(ret) = ret {
                            error!("{:?}", ret);
                        }
                    }
                },
                Err(error) => {
                    return Err(anyhow!("error listening in wasm thread: {:?}", error));
                }
            }
        }
    }

    pub fn run(
        &mut self,
        data: Vec<u8>,
        sender: SyncSender<Result<()>>,
        ct: CancellationToken,
    ) -> Result<()> {
        let _guard = OnDrop(|| (self.on_stopped_running)());

        let module = Module::from_vec(self.runtime.clone(), data, "env")
            .map_err(|e| anyhow::anyhow!("failed to create module: {e:?}"))?;

        info!("Installing wasm instance");
        let instance = Rc::new(
            Instance::new(module.into(), 1024 * 16)
                .map_err(|e| format!("failed to create instance: {}", e)) // TODO these should be more specific types for wasm and no unwrap AND CLEANUP TOO
                .unwrap(), // TODO: Remove this unwrap.
        );

        info!("running wasm");
        self.active_wasm_instance
            .store(instance.get_inner_instance() as usize, Ordering::Relaxed);

        self.last_host_fn_call_ticks_ms
            .store(unsafe { xTaskGetTickCount() }, Ordering::Relaxed);

        let mut wasm_data = Box::new(WasmData {
            ct,
            hardware: self.hardware.clone(),
            last_host_fn_call_ticks_ms: self.last_host_fn_call_ticks_ms.clone(),
        });

        unsafe {
            wasm_runtime_set_custom_data(
                instance.get_inner_instance(),
                wasm_data.as_mut() as *mut WasmData as *mut c_void,
            )
        }; //  TODO: disable WAMR_BUILD_LIB_PTHREAD

        info!("Finding main");

        let main_function = Function::find_export_func(instance.clone(), "main").unwrap();

        // TODO: here send response running

        sender.send(anyhow::Ok(())).unwrap();
        let res = main_function.call(&vec![]);
        log::info!("{:?}", res);
        self.active_wasm_instance.store(0, Ordering::Relaxed);

        // cleanup after cancellation
        if wasm_data.ct.is_cancelled() {
            log::info!("cancelled");
        }

        {
            wasm_data.hardware.stop_all();
        }

        log::info!("done");
        Ok(())
    }
}

struct OnDrop<F: Fn()>(F);

impl<F: Fn()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
