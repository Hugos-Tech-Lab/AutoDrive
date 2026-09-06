wit_bindgen::generate!({
    world: "app",
    path: "wit"
});

pub mod the_beginner;

#[macro_export]
macro_rules! register {
    ($app:ty) => {
        struct __TheBeginnerAdapter;

        impl $crate::Guest for __TheBeginnerAdapter {
            fn run() {
                <$app as $crate::TheBeginnerAutoDrive>::run();
            }
        }

        $crate::export!(__TheBeginnerAdapter);
    };
}


#[allow(dead_code)]
struct Application;

impl Guest for Application {
    #[allow(async_fn_in_trait)]
    fn run() -> () {
        // let temp = temperature::temperature();
        // temperature::log(&format!("{temp}"));
        // temperature::log(&format!("{temp}"));
        // temperature::log(&format!("{temp}"));
    }
    
    #[allow(async_fn_in_trait)]
    fn prepare() -> () {
        todo!()
    }
}


export!(Application);
