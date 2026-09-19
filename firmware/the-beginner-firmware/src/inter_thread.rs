use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_sync::signal::Signal;

pub struct InterTaskProducer<'a, Mtx: RawMutex, M, R, const N: usize> {
    sender: Sender<'a, Mtx, (M, &'a Signal<Mtx, R>), N>,
    // Dedicated signal for this producer to receive the response.
    // Pre-allocating this avoids dynamic heap allocation per request.
    response_signal: &'a Signal<Mtx, R>,
}

impl<'a, Mtx: RawMutex, M, R, const N: usize> InterTaskProducer<'a, Mtx, M, R, N> {
    #[must_use]
    // We take `&mut self` here so you cannot accidentally fire multiple 
    // concurrent requests from the SAME producer, which would overwrite the signal.
    pub async fn send(&mut self, message: M) -> R {
        // Reset the signal to clear any stale state before we wait on it
        self.response_signal.reset();

        // Send the message together with the reference to our response signal
        self.sender
            .send((message, self.response_signal))
            .await;

        // Wait for the listener to process the message and signal back
        self.response_signal.wait().await
    }
}

pub struct InterTaskResponse<'a, Mtx: RawMutex, R> {
    response_signal: &'a Signal<Mtx, R>,
    sent: bool,
}

impl<'a, Mtx: RawMutex, R> InterTaskResponse<'a, Mtx, R> {
    pub fn send(mut self, response: R) {
        if !self.sent {
            self.response_signal.signal(response);
        }
        self.sent = true;
    }
}

impl<'a, Mtx: RawMutex, R> Drop for InterTaskResponse<'a, Mtx, R> {
    fn drop(&mut self) {
        if !self.sent {
            // Note: Panicking in `drop` behaves the same as standard Rust, but in 
            // embedded contexts you might want to log this via `defmt::error!` instead.
            panic!("Dropped response without sending any message");
        }
    }
}

pub struct InterTaskListener<'a, Mtx: RawMutex, M, R, const N: usize> {
    receiver: Receiver<'a, Mtx, (M, &'a Signal<Mtx, R>), N>,
}

impl<'a, Mtx: RawMutex, M, R, const N: usize> InterTaskListener<'a, Mtx, M, R, N> {
    // Embassy channels don't have the concept of a "disconnected" error natively
    // because they are statically allocated. We can safely remove `anyhow::Result`.
    pub async fn listen(&self) -> (M, InterTaskResponse<'a, Mtx, R>) {
        let (message, response_signal) = self.receiver.receive().await;
        (message, InterTaskResponse { response_signal, sent: false })
    }
}

// In Embassy, initialization requires passing in the statically allocated primitives.
pub fn create<'a, Mtx: RawMutex, M, R, const N: usize>(
    channel: &'a Channel<Mtx, (M, &'a Signal<Mtx, R>), N>,
    signal: &'a Signal<Mtx, R>,
) -> (InterTaskProducer<'a, Mtx, M, R, N>, InterTaskListener<'a, Mtx, M, R, N>) {
    (
        InterTaskProducer {
            sender: channel.sender(),
            response_signal: signal,
        },
        InterTaskListener {
            receiver: channel.receiver(),
        },
    )
}