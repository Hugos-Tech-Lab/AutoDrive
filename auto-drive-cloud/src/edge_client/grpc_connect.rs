use log::warn;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tonic::{Response, Status};

pub async fn grpc_connect<C, Req, Res, F>(mut client: C, request: Req, mut rpc_fn: F) -> Res
where
    Req: Clone,
    F: AsyncFnMut(&mut C, Req) -> Result<Response<Res>, Status>,
{
    let rpc_name = std::any::type_name::<F>();

    loop {
        let result = timeout(Duration::from_secs(5), rpc_fn(&mut client, request.clone())).await;
        let result = match result {
            Ok(result) => result,
            Err(_) => {
                warn!("[{rpc_name}] Timed out");
                continue;
            }
        };

        match result {
            Ok(response) => break response.into_inner(),
            Err(err) => {
                warn!("[{rpc_name}] Failed to connect via gRPC: {err}. Retrying in 5s...");
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
    }
}
