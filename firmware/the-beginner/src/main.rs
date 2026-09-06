use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker};
use wasmtime::component::{HasSelf, ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    world: "app",
    path: "interface/the-beginner.wit",
});

struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl example::device::hardware::Host for HostState {
    fn control_wheel(&mut self,wheel: f32,speed: f32,) -> () {
        todo!()
    }

    fn read_wheel(&mut self,) -> f32 {
        todo!()
    }
}

fn main() -> anyhow::Result<()> {
    let mut config = Config::new();

    config.wasm_component_model_implements(true);

    let engine = Engine::new(&config)?;

    let component = Component::from_file(
        &engine,
        "../../examples/the-beginner/target/wasm32-wasip2/debug/the_beginner.wasm",
    )?;

    let mut linker = Linker::new(&engine);

    // Add WASI implementations.
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // Add our custom WIT interface.
    App::add_to_linker::<_, HasSelf<_>>(
        &mut linker,
        |state| state,
    )?;

    let wasi = WasiCtx::builder().build();

    let mut store = Store::new(
        &engine,
        HostState {
            wasi,
            table: ResourceTable::new(),
        },
    );

    let app = App::instantiate(
        &mut store,
        &component,
        &linker,
    )?;

    app.call_run(&mut store)?;

    Ok(())
}