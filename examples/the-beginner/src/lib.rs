wit_bindgen::generate!({
    world: "app",
    path: "wit"
});

#[allow(dead_code)]
struct App;

impl Guest for App {
    #[allow(async_fn_in_trait)]
    fn run() -> () {
        let temp = temperature::temperature();
        temperature::log(&format!("{temp}"));
        temperature::log(&format!("{temp}"));
        temperature::log(&format!("{temp}"));
    }
}

export!(App);
