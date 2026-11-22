use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::sync::{broadcast, mpsc, watch};

#[derive(Debug)]
pub struct TheBeginnerCarConnectionHandle {
    connected_rx: watch::Receiver<bool>,
    outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
}

impl Clone for TheBeginnerCarConnectionHandle {
    fn clone(&self) -> Self {
        Self {
            connected_rx: self.connected_rx.clone(),
            outgoing_tx: self.outgoing_tx.clone(),
            incoming_rx: self.incoming_rx.resubscribe(),
        }
    }
}

impl TheBeginnerCarConnectionHandle {
    pub fn new() {
        //
    }
}
