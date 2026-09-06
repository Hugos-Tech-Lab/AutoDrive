use the_beginner::{register, the_beginner_auto_drive::TheBeginnerAutoDrive};

struct BasicAutoDrive;

impl TheBeginnerAutoDrive for BasicAutoDrive {
    fn run() {

        // the_beginner::
        let a = the_beginner::read_wheel();
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
