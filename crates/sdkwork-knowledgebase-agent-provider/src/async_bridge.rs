use std::future::Future;
use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;
use std::thread;

type BridgeJob = Box<dyn FnOnce(&tokio::runtime::Runtime) + Send>;

struct AsyncBridge {
    job_tx: Sender<BridgeJob>,
}

static ASYNC_BRIDGE: OnceLock<AsyncBridge> = OnceLock::new();

fn bridge() -> &'static AsyncBridge {
    ASYNC_BRIDGE.get_or_init(|| {
        let (job_tx, job_rx) = mpsc::channel::<BridgeJob>();
        thread::Builder::new()
            .name("kb-async-bridge".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                    .expect("async bridge runtime");
                for job in job_rx {
                    job(&runtime);
                }
            })
            .expect("async bridge thread");
        AsyncBridge { job_tx }
    })
}

/// Run an async client call from the sync `KnowledgeProvider` surface.
///
/// Jobs execute on a single dedicated bridge thread with a reused Tokio runtime so
/// provider search never deadlocks the hosting Axum worker and does not spawn a
/// new OS thread per call.
pub fn block_on_async<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    bridge()
        .job_tx
        .send(Box::new(move |runtime| {
            let result = runtime.block_on(future);
            let _ = result_tx.send(result);
        }))
        .expect("async bridge enqueue");
    result_rx.recv().expect("async bridge response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn block_on_async_does_not_deadlock_inside_multi_thread_runtime() {
        let value = tokio::task::spawn_blocking(|| block_on_async(async { 42 }))
            .await
            .expect("join");

        assert_eq!(value, 42);
    }
}
