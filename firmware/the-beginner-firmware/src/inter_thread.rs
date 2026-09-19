use std::sync::mpsc;

pub struct InterThreadProducer<M, R> {
    sender: mpsc::Sender<(M, mpsc::Sender<R>)>,
}

impl<M, R> InterThreadProducer<M, R> {
    #[must_use]
    pub fn send(&self, message: M) -> R {
        // Channel dedicated to this particular request.
        let (response_sender, response_receiver) = mpsc::channel();

        // Send the message together with the channel on which we expect
        // the response.
        self.sender
            .send((message, response_sender))
            .expect("listener thread has stopped");

        // Wait for the listener to process the message.
        response_receiver
            .recv()
            .expect("listener thread has stopped")
    }
}

pub struct InterThreadResponse<R> {
    response_sender: mpsc::Sender<R>,
    sent: bool,
}

impl<R> InterThreadResponse<R> {
    pub fn send(mut self, response: R) {
        if !self.sent {
            self.response_sender
                .send(response)
                .expect("producer thread has stopped");
        }

        self.sent = true;
    }
}

impl<R> Drop for InterThreadResponse<R> {
    fn drop(&mut self) {
        if !self.sent {
            panic!("Dropped response without sending any message");
        }
    }
}

pub struct InterThreadListener<M, R> {
    receiver: mpsc::Receiver<(M, mpsc::Sender<R>)>,
}

impl<M, R> InterThreadListener<M, R> {
    pub fn listen(&self) -> anyhow::Result<(M, InterThreadResponse<R>)> {
      let (message, response_sender) = self.receiver.recv()?;
        Ok((message, InterThreadResponse { response_sender, sent: false }))
    }
}

pub fn create<M, R>() -> (InterThreadProducer<M, R>, InterThreadListener<M, R>) {
    let (sender, receiver) = mpsc::channel();

    (
        InterThreadProducer { sender },
        InterThreadListener { receiver },
    )
}
