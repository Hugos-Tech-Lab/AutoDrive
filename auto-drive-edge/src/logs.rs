use std::sync::Arc;

use auto_drive_common_types::LogMessage;
use futures_util::{Stream, stream};
use tokio::sync::{RwLock, broadcast};
use tokio_stream::{
    StreamExt,
    wrappers::{BroadcastStream, errors::BroadcastStreamRecvError},
};

#[derive(Clone, Debug)]
pub struct Logs {
    history: Arc<RwLock<Vec<LogMessage>>>,
    live: broadcast::Sender<LogMessage>,
}

impl Logs {
    pub fn new() -> Self {
        Self {
            history: Default::default(),
            live: broadcast::Sender::<LogMessage>::new(100),
        }
    }

    pub async fn push(&self, log_message: LogMessage) {
        let mut history_lock = self.history.write().await;
        let _ = self.live.send(log_message.clone());
        history_lock.push(log_message);
    }

    pub async fn stream_with_history(
        &self,
    ) -> impl Stream<Item = Result<LogMessage, BroadcastStreamRecvError>> + Send + Sync + 'static
    {
        let live_rx;
        let history;
        {
            let history_guard = self.history.read().await;
            live_rx = self.live.subscribe();
            history = history_guard.clone();
        }

        let history_stream = stream::iter(history.into_iter().map(Ok));
        let live_stream = BroadcastStream::new(live_rx);

        history_stream.chain(live_stream)
    }
}
