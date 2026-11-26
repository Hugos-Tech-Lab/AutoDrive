use the_beginner_car_tcp_protocol::{AutoScript, TheBeginnerCarIncomingMessages};

use crate::edge_connection::tcp_server::TcpServer;

pub mod tcp_server;

#[derive(Debug, Clone)]
pub struct EdgeConnection {
    tcp_server: TcpServer,
    // vehicle: Arc
}

impl EdgeConnection {
    pub fn new() -> Self {
        let tcp_server = TcpServer::new();
        // let tcp_server_receiver = tcp_server.receiver();

        Self { tcp_server }
    }

    pub fn handle_incoming_message(message: TheBeginnerCarIncomingMessages) {
        match message {
            TheBeginnerCarIncomingMessages::NotifyPositionFromCamera {
                x,
                y,
                rotation_radians,
                timestamp_unix_epoch_ms,
            } => todo!(),
            TheBeginnerCarIncomingMessages::CancelAutoScript { run_uuid } => todo!(),
            TheBeginnerCarIncomingMessages::RunAutoScript { script } => todo!(),
        }
    }

    fn run_auto_script(auto_script: AutoScript) {
        //
    }
}
