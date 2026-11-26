use the_beginner_car_tcp_protocol::AutoScript;

#[derive(Debug)]
pub struct Vehicle {
    auto_script: Option<AutoScript>,
}

impl Vehicle {
    pub fn run_auto_script(&self, auto_script: AutoScript) {
        if self.auto_script.is_some() {
            return; // TODO: return error
        }

        //
    }
}
