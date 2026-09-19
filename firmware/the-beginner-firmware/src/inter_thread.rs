pub struct InterThreadProducer<M, R> {
    sender: flume::Sender<(M, flume::Sender<R>)>,
}

impl<M, R> InterThreadProducer<M, R> {
    /// Synchronous (blocking) send
    #[must_use]
    pub fn send(&self, message: M) -> R {
        // Use bounded(1) since we only ever expect exactly one response
        let (response_sender, response_receiver) = flume::bounded(1);

        self.sender
            .send((message, response_sender))
            .expect("listener thread has stopped");

        response_receiver
            .recv()
            .expect("listener thread has stopped")
    }

    /// Asynchronous (non-blocking) send for use in async contexts
    #[must_use]
    pub async fn send_async(&self, message: M) -> R {
        let (response_sender, response_receiver) = flume::bounded(1);

        self.sender
            .send_async((message, response_sender))
            .await
            .expect("listener thread has stopped");

        response_receiver
            .recv_async()
            .await
            .expect("listener thread has stopped")
    }
}

pub struct InterThreadResponse<R> {
    response_sender: flume::Sender<R>,
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
    receiver: flume::Receiver<(M, flume::Sender<R>)>,
}

impl<M, R> InterThreadListener<M, R> {
    /// Synchronous (blocking) listen
    pub fn listen(&self) -> anyhow::Result<(M, InterThreadResponse<R>)> {
        // flume's RecvError trivially converts to anyhow::Error with `?`
        let (message, response_sender) = self.receiver.recv()?;
        Ok((
            message,
            InterThreadResponse {
                response_sender,
                sent: false,
            },
        ))
    }

    /// Asynchronous (non-blocking) listen for use in async contexts
    pub async fn listen_async(&self) -> anyhow::Result<(M, InterThreadResponse<R>)> {
        let (message, response_sender) = self.receiver.recv_async().await?;
        Ok((
            message,
            InterThreadResponse {
                response_sender,
                sent: false,
            },
        ))
    }
}

pub fn create<M, R>() -> (InterThreadProducer<M, R>, InterThreadListener<M, R>) {
    // std::sync::mpsc::channel() is unbounded, so we use flume::unbounded()
    // Alternatively, you could use flume::bounded(16) if you want backpressure.
    let (sender, receiver) = flume::unbounded();

    (
        InterThreadProducer { sender },
        InterThreadListener { receiver },
    )
}
