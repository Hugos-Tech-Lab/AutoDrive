use auto_drive_sdk::the_beginner::{register, TheBeginnerAutoDrive};

struct BasicAutoDrive;

impl TheBeginnerAutoDrive for BasicAutoDrive {
    fn run() {
        // auto_drive_the_beginner_sdk::control_wheel(wheel, speed);
        // the_beginner::
        // let a = read_wheel();
        // let temperature = temperature();

        // log(&format!(
        //     "Current temperature: {temperature} °C"
        // ));

        // if temperature > 100.0 {
        //     set_throttle(0.0);
        // } else {
        //     set_throttle(50.0);
        // }
    }
    
    fn run_2() -> () {
        todo!()
    }
}

register!(BasicAutoDrive);
