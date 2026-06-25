use std::future::Future;

/// Run an async client call from the sync `KnowledgeProvider` surface.
///
/// Always executes on a dedicated thread with its own current-thread runtime so
/// provider search never deadlocks the hosting Tokio worker (for example Axum
/// handlers or `#[tokio::test]` workers invoking sync kernel providers).
pub fn block_on_async<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("async bridge runtime")
            .block_on(future)
    })
    .join()
    .expect("async bridge thread")
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
