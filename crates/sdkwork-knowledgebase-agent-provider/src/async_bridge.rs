use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

type BridgeJob = Box<dyn FnOnce(&tokio::runtime::Handle) + Send>;

const BRIDGE_START_TIMEOUT: Duration = Duration::from_secs(5);
const BRIDGE_SHUTDOWN_GRACE_TIMEOUT: Duration = Duration::from_millis(250);
const BRIDGE_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
const BRIDGE_SHUTDOWN_COMPLETION_MARGIN: Duration = Duration::from_secs(1);

/// Maximum number of jobs accepted by the bridge, including in-flight jobs.
pub const ASYNC_BRIDGE_QUEUE_CAPACITY: usize = 64;

/// Default deadline for enqueue-to-result completion on the synchronous surface.
pub const ASYNC_BRIDGE_TIMEOUT: Duration = Duration::from_secs(30);

/// Failures produced by the synchronous-to-asynchronous bridge boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AsyncBridgeError {
    InvalidQueueCapacity,
    InitializationFailed { message: String },
    ThreadSpawnFailed { message: String },
    QueueSaturated { capacity: usize },
    TimedOut { timeout: Duration },
    ShuttingDown,
    TaskPanicked,
    WorkerStopped,
    WorkerPanicked,
    StateUnavailable,
    ShutdownFailed,
    ShutdownTimedOut { timeout: Duration },
}

impl std::fmt::Display for AsyncBridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidQueueCapacity => {
                formatter.write_str("async bridge queue capacity must be greater than zero")
            }
            Self::InitializationFailed { message } => {
                write!(formatter, "async bridge initialization failed: {message}")
            }
            Self::ThreadSpawnFailed { message } => {
                write!(formatter, "async bridge thread spawn failed: {message}")
            }
            Self::QueueSaturated { capacity } => {
                write!(
                    formatter,
                    "async bridge queue is saturated at capacity {capacity}"
                )
            }
            Self::TimedOut { timeout } => {
                write!(
                    formatter,
                    "async bridge request timed out after {timeout:?}"
                )
            }
            Self::ShuttingDown => formatter.write_str("async bridge is shutting down"),
            Self::TaskPanicked => formatter.write_str("async bridge task panicked"),
            Self::WorkerStopped => formatter.write_str("async bridge worker stopped"),
            Self::WorkerPanicked => formatter.write_str("async bridge worker panicked"),
            Self::StateUnavailable => formatter.write_str("async bridge state is unavailable"),
            Self::ShutdownFailed => formatter.write_str("async bridge shutdown failed"),
            Self::ShutdownTimedOut { timeout } => {
                write!(
                    formatter,
                    "async bridge shutdown timed out after {timeout:?}"
                )
            }
        }
    }
}

impl std::error::Error for AsyncBridgeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BridgeState {
    Running,
    ShuttingDown,
    Stopped,
}

struct BridgeLifecycle {
    job_tx: Option<SyncSender<BridgeJob>>,
    worker: Option<JoinHandle<()>>,
    worker_completion: Option<Receiver<Result<(), AsyncBridgeError>>>,
    state: BridgeState,
    shutdown_result: Option<Result<(), AsyncBridgeError>>,
}

struct AsyncBridge {
    capacity: usize,
    admission: Arc<tokio::sync::Semaphore>,
    lifecycle: Mutex<BridgeLifecycle>,
    shutdown_completed: Condvar,
    shutdown_wait_timeout: Duration,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

struct PendingBridgeResult<T> {
    result_rx: Receiver<Result<T, AsyncBridgeError>>,
}

impl<T> PendingBridgeResult<T> {
    fn wait(self, timeout: Duration) -> Result<T, AsyncBridgeError> {
        match self.result_rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => Err(AsyncBridgeError::TimedOut { timeout }),
            Err(RecvTimeoutError::Disconnected) => Err(AsyncBridgeError::WorkerStopped),
        }
    }
}

impl AsyncBridge {
    fn start(capacity: usize) -> Result<Self, AsyncBridgeError> {
        Self::start_with_shutdown_timeouts(
            capacity,
            BRIDGE_SHUTDOWN_GRACE_TIMEOUT,
            BRIDGE_SHUTDOWN_TIMEOUT,
        )
    }

    fn start_with_shutdown_timeouts(
        capacity: usize,
        shutdown_grace_timeout: Duration,
        runtime_shutdown_timeout: Duration,
    ) -> Result<Self, AsyncBridgeError> {
        if capacity == 0 || capacity > tokio::sync::Semaphore::MAX_PERMITS {
            return Err(AsyncBridgeError::InvalidQueueCapacity);
        }
        let admission_permits =
            u32::try_from(capacity).map_err(|_| AsyncBridgeError::InvalidQueueCapacity)?;

        let (job_tx, job_rx) = mpsc::sync_channel::<BridgeJob>(capacity);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (worker_completion_tx, worker_completion_rx) = mpsc::sync_channel(1);
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);
        let worker_shutdown_tx = shutdown_tx.clone();
        let admission = Arc::new(tokio::sync::Semaphore::new(capacity));
        let worker_admission = Arc::clone(&admission);
        let shutdown_timeout = shutdown_grace_timeout.saturating_add(runtime_shutdown_timeout);
        let shutdown_wait_timeout =
            shutdown_timeout.saturating_add(BRIDGE_SHUTDOWN_COMPLETION_MARGIN);
        let worker = thread::Builder::new()
            .name("kb-async-bridge".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let runtime = match tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build()
                    {
                        Ok(runtime) => runtime,
                        Err(error) => {
                            let message = error.to_string();
                            let _ = ready_tx.send(Err(message.clone()));
                            return Err(AsyncBridgeError::InitializationFailed { message });
                        }
                    };
                    if ready_tx.send(Ok(())).is_err() {
                        return Ok(());
                    }
                    let handle = runtime.handle().clone();
                    for job in job_rx {
                        job(&handle);
                    }
                    let graceful_admission = Arc::clone(&worker_admission);
                    let graceful = runtime.block_on(async move {
                        match tokio::time::timeout(
                            shutdown_grace_timeout,
                            graceful_admission.acquire_many_owned(admission_permits),
                        )
                        .await
                        {
                            Ok(Ok(permits)) => {
                                drop(permits);
                                true
                            }
                            _ => false,
                        }
                    });
                    let remaining_runtime_timeout = if graceful {
                        runtime_shutdown_timeout
                    } else {
                        worker_shutdown_tx.send_replace(true);
                        let forced_shutdown_started = Instant::now();
                        let _ = runtime.block_on(async {
                            tokio::time::timeout(
                                runtime_shutdown_timeout,
                                worker_admission.acquire_many_owned(admission_permits),
                            )
                            .await
                        });
                        runtime_shutdown_timeout.saturating_sub(forced_shutdown_started.elapsed())
                    };
                    runtime.shutdown_timeout(remaining_runtime_timeout);
                    if graceful {
                        Ok(())
                    } else {
                        Err(AsyncBridgeError::ShutdownTimedOut {
                            timeout: shutdown_timeout,
                        })
                    }
                }))
                .unwrap_or(Err(AsyncBridgeError::WorkerPanicked));
                let _ = worker_completion_tx.send(result);
            })
            .map_err(|error| AsyncBridgeError::ThreadSpawnFailed {
                message: error.to_string(),
            })?;

        match ready_rx.recv_timeout(BRIDGE_START_TIMEOUT) {
            Ok(Ok(())) => Ok(Self {
                capacity,
                admission,
                lifecycle: Mutex::new(BridgeLifecycle {
                    job_tx: Some(job_tx),
                    worker: Some(worker),
                    worker_completion: Some(worker_completion_rx),
                    state: BridgeState::Running,
                    shutdown_result: None,
                }),
                shutdown_completed: Condvar::new(),
                shutdown_wait_timeout,
                shutdown_tx,
            }),
            Ok(Err(message)) => {
                drop(job_tx);
                if worker_completion_rx
                    .recv_timeout(shutdown_wait_timeout)
                    .is_ok()
                {
                    let _ = worker.join();
                }
                Err(AsyncBridgeError::InitializationFailed { message })
            }
            Err(error) => {
                drop(job_tx);
                if worker_completion_rx
                    .recv_timeout(shutdown_wait_timeout)
                    .is_ok()
                {
                    let _ = worker.join();
                }
                Err(AsyncBridgeError::InitializationFailed {
                    message: error.to_string(),
                })
            }
        }
    }

    fn submit<F, T>(&self, future: F) -> Result<PendingBridgeResult<T>, AsyncBridgeError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| AsyncBridgeError::StateUnavailable)?;
        if lifecycle.state != BridgeState::Running {
            return Err(AsyncBridgeError::ShuttingDown);
        }
        let admission_permit = Arc::clone(&self.admission)
            .try_acquire_owned()
            .map_err(|_| AsyncBridgeError::QueueSaturated {
                capacity: self.capacity,
            })?;
        let (result_tx, result_rx) = mpsc::sync_channel(1);
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let job: BridgeJob = Box::new(move |runtime| {
            let task = runtime.spawn(async move {
                if *shutdown_rx.borrow() {
                    Err(AsyncBridgeError::ShuttingDown)
                } else {
                    tokio::select! {
                        biased;
                        _ = shutdown_rx.changed() => Err(AsyncBridgeError::ShuttingDown),
                        result = future => Ok(result),
                    }
                }
            });
            runtime.spawn(async move {
                let result = match task.await {
                    Ok(result) => result,
                    Err(error) if error.is_panic() => Err(AsyncBridgeError::TaskPanicked),
                    Err(_) => Err(AsyncBridgeError::WorkerStopped),
                };
                let _ = result_tx.send(result);
                drop(admission_permit);
            });
        });
        let Some(job_tx) = lifecycle.job_tx.as_ref() else {
            return Err(AsyncBridgeError::ShuttingDown);
        };
        job_tx.try_send(job).map_err(|error| match error {
            TrySendError::Full(_) => AsyncBridgeError::QueueSaturated {
                capacity: self.capacity,
            },
            TrySendError::Disconnected(_) => AsyncBridgeError::WorkerStopped,
        })?;
        Ok(PendingBridgeResult { result_rx })
    }

    fn shutdown(&self) -> Result<(), AsyncBridgeError> {
        let (worker, worker_completion) = {
            let mut lifecycle = self
                .lifecycle
                .lock()
                .map_err(|_| AsyncBridgeError::StateUnavailable)?;
            loop {
                match lifecycle.state {
                    BridgeState::Running => {
                        lifecycle.state = BridgeState::ShuttingDown;
                        lifecycle.job_tx.take();
                        break (lifecycle.worker.take(), lifecycle.worker_completion.take());
                    }
                    BridgeState::ShuttingDown => {
                        lifecycle = self
                            .shutdown_completed
                            .wait(lifecycle)
                            .map_err(|_| AsyncBridgeError::StateUnavailable)?;
                    }
                    BridgeState::Stopped => {
                        return lifecycle
                            .shutdown_result
                            .clone()
                            .unwrap_or(Err(AsyncBridgeError::ShutdownFailed));
                    }
                }
            }
        };

        let result = match (worker, worker_completion) {
            (Some(worker), Some(worker_completion)) => {
                match worker_completion.recv_timeout(self.shutdown_wait_timeout) {
                    Ok(worker_result) => match worker.join() {
                        Ok(()) => worker_result,
                        Err(_) => Err(AsyncBridgeError::WorkerPanicked),
                    },
                    Err(RecvTimeoutError::Timeout) => {
                        drop(worker);
                        Err(AsyncBridgeError::ShutdownTimedOut {
                            timeout: self.shutdown_wait_timeout,
                        })
                    }
                    Err(RecvTimeoutError::Disconnected) => match worker.join() {
                        Ok(()) => Err(AsyncBridgeError::WorkerStopped),
                        Err(_) => Err(AsyncBridgeError::WorkerPanicked),
                    },
                }
            }
            _ => Err(AsyncBridgeError::ShutdownFailed),
        };

        let mut lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| AsyncBridgeError::StateUnavailable)?;
        lifecycle.state = BridgeState::Stopped;
        lifecycle.shutdown_result = Some(result.clone());
        self.shutdown_completed.notify_all();
        result
    }
}

impl Drop for AsyncBridge {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

static ASYNC_BRIDGE: OnceLock<Result<AsyncBridge, AsyncBridgeError>> = OnceLock::new();
static ASYNC_BRIDGE_INIT_LOCK: Mutex<()> = Mutex::new(());
static ASYNC_BRIDGE_SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

fn bridge() -> Result<&'static AsyncBridge, AsyncBridgeError> {
    if ASYNC_BRIDGE_SHUTDOWN_REQUESTED.load(AtomicOrdering::Acquire) {
        return Err(AsyncBridgeError::ShuttingDown);
    }
    let _initialization = ASYNC_BRIDGE_INIT_LOCK
        .lock()
        .map_err(|_| AsyncBridgeError::StateUnavailable)?;
    if ASYNC_BRIDGE_SHUTDOWN_REQUESTED.load(AtomicOrdering::Acquire) {
        return Err(AsyncBridgeError::ShuttingDown);
    }
    let initialized = ASYNC_BRIDGE.get_or_init(|| AsyncBridge::start(ASYNC_BRIDGE_QUEUE_CAPACITY));
    if ASYNC_BRIDGE_SHUTDOWN_REQUESTED.load(AtomicOrdering::Acquire) {
        if let Ok(bridge) = initialized {
            let _ = bridge.shutdown();
        }
        return Err(AsyncBridgeError::ShuttingDown);
    }
    match initialized {
        Ok(bridge) => Ok(bridge),
        Err(error) => Err(error.clone()),
    }
}

/// Run an async client call from the sync `KnowledgeProvider` surface.
///
/// Jobs execute on one reused Tokio runtime behind bounded admission so provider
/// search does not block the hosting Axum worker or create one runtime per call.
pub fn block_on_async<F, T>(future: F) -> Result<T, AsyncBridgeError>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    block_on_async_with_timeout(future, ASYNC_BRIDGE_TIMEOUT)
}

/// Run an async client call with an explicit synchronous wait deadline.
pub fn block_on_async_with_timeout<F, T>(
    future: F,
    timeout: Duration,
) -> Result<T, AsyncBridgeError>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    bridge()
        .and_then(|bridge| bridge.submit(future))
        .and_then(|result| result.wait(timeout))
}

/// Stop the process-wide bridge, cancel accepted work, and wait for its worker.
pub fn shutdown_async_bridge() -> Result<(), AsyncBridgeError> {
    ASYNC_BRIDGE_SHUTDOWN_REQUESTED.store(true, AtomicOrdering::Release);
    let _initialization = ASYNC_BRIDGE_INIT_LOCK
        .lock()
        .map_err(|_| AsyncBridgeError::StateUnavailable)?;
    match ASYNC_BRIDGE.get() {
        Some(Ok(bridge)) => bridge.shutdown(),
        Some(Err(error)) => Err(error.clone()),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    struct DropNotifier(Option<SyncSender<()>>);

    impl Drop for DropNotifier {
        fn drop(&mut self) {
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }

    #[test]
    fn capacity_is_the_exact_accepted_job_limit() {
        let bridge = AsyncBridge::start(1).expect("start bridge");
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();

        let first_result = bridge
            .submit(async move {
                started_tx.send(()).expect("report started");
                let _ = release_rx.await;
                1_u8
            })
            .expect("submit active job");
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("active job started");

        let error = match bridge.submit(async { 2_u8 }) {
            Ok(_) => panic!("second job must not exceed accepted-job capacity"),
            Err(error) => error,
        };

        assert_eq!(error, AsyncBridgeError::QueueSaturated { capacity: 1 });

        release_tx.send(()).expect("release active job");
        assert_eq!(first_result.wait(Duration::from_secs(1)), Ok(1_u8));
        assert_eq!(
            bridge
                .submit(async { 2_u8 })
                .expect("submit after capacity is released")
                .wait(Duration::from_secs(1)),
            Ok(2_u8)
        );
        bridge.shutdown().expect("shutdown bridge");
    }

    #[test]
    fn wait_timeout_keeps_accepted_future_admitted_until_it_finishes() {
        let bridge = AsyncBridge::start(1).expect("start bridge");
        let (blocking_started_tx, blocking_started_rx) = mpsc::sync_channel(1);
        let (future_completed_tx, future_completed_rx) = mpsc::sync_channel(1);
        let (release_blocking_tx, release_blocking_rx) = mpsc::sync_channel(1);
        let timeout = Duration::from_millis(25);

        let pending = bridge
            .submit(async move {
                tokio::task::spawn_blocking(move || {
                    blocking_started_tx
                        .send(())
                        .expect("report blocking task started");
                    let _ = release_blocking_rx.recv();
                })
                .await
                .expect("join blocking task");
                future_completed_tx
                    .send(())
                    .expect("report accepted future completion");
            })
            .expect("submit accepted future");
        blocking_started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("blocking task started");

        assert_eq!(
            pending.wait(timeout),
            Err(AsyncBridgeError::TimedOut { timeout })
        );
        thread::sleep(Duration::from_millis(50));
        let saturation_error = match bridge.submit(async { 2_u8 }) {
            Ok(result) => {
                let _ = result.wait(Duration::from_secs(1));
                None
            }
            Err(error) => Some(error),
        };

        release_blocking_tx.send(()).expect("release blocking task");
        let future_completion = future_completed_rx.recv_timeout(Duration::from_secs(1));
        let follow_up_deadline = Instant::now() + Duration::from_secs(1);
        let follow_up = loop {
            match bridge.submit(async { 3_u8 }) {
                Ok(result) => break result.wait(Duration::from_secs(1)),
                Err(AsyncBridgeError::QueueSaturated { .. })
                    if Instant::now() < follow_up_deadline =>
                {
                    thread::yield_now();
                }
                Err(error) => break Err(error),
            }
        };
        let shutdown_result = bridge.shutdown();

        assert_eq!(
            saturation_error,
            Some(AsyncBridgeError::QueueSaturated { capacity: 1 }),
            "caller timeout must not detach accepted work from admission"
        );
        assert_eq!(future_completion, Ok(()));
        assert_eq!(follow_up, Ok(3_u8));
        assert_eq!(shutdown_result, Ok(()));
    }

    #[test]
    fn wait_timeout_does_not_cancel_the_accepted_future() {
        let bridge = AsyncBridge::start(1).expect("start bridge");
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (dropped_tx, dropped_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        let timeout = Duration::from_millis(25);

        let result = bridge
            .submit(async move {
                let _drop_notifier = DropNotifier(Some(dropped_tx));
                started_tx.send(()).expect("report started");
                let _ = release_rx.await;
                1_u8
            })
            .expect("submit pending job");
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("pending job started");

        assert_eq!(
            result.wait(timeout),
            Err(AsyncBridgeError::TimedOut { timeout })
        );

        assert_eq!(
            dropped_rx.recv_timeout(Duration::from_millis(50)),
            Err(RecvTimeoutError::Timeout),
            "caller timeout must not cancel accepted work"
        );
        assert_eq!(
            match bridge.submit(async { 2_u8 }) {
                Ok(_) => panic!("accepted work must retain admission after caller timeout"),
                Err(error) => error,
            },
            AsyncBridgeError::QueueSaturated { capacity: 1 }
        );

        release_tx.send(()).expect("release accepted future");
        dropped_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("accepted future completed");
        let follow_up_deadline = Instant::now() + Duration::from_secs(1);
        loop {
            match bridge.submit(async { 2_u8 }) {
                Ok(result) => {
                    assert_eq!(result.wait(Duration::from_secs(1)), Ok(2_u8));
                    break;
                }
                Err(AsyncBridgeError::QueueSaturated { .. })
                    if Instant::now() < follow_up_deadline =>
                {
                    thread::yield_now();
                }
                Err(error) => panic!("submit follow-up job: {error}"),
            }
        }
        bridge.shutdown().expect("shutdown bridge");
    }

    #[test]
    fn dropping_result_receiver_does_not_cancel_accepted_work() {
        let bridge = AsyncBridge::start(1).expect("start bridge");
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        let (completed_tx, completed_rx) = mpsc::sync_channel(1);
        let pending = bridge
            .submit(async move {
                started_tx.send(()).expect("report started");
                let _ = release_rx.await;
                completed_tx.send(()).expect("report completion");
            })
            .expect("submit accepted job");
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("accepted job started");
        drop(pending);

        assert_eq!(
            match bridge.submit(async { 2_u8 }) {
                Ok(_) => panic!("dropped receiver must not release admission"),
                Err(error) => error,
            },
            AsyncBridgeError::QueueSaturated { capacity: 1 }
        );
        release_tx.send(()).expect("release accepted job");
        completed_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("dropped receiver did not cancel accepted work");
        bridge.shutdown().expect("shutdown bridge");
    }

    #[test]
    fn concurrent_shutdown_callers_observe_the_same_terminal_state() {
        let bridge = Arc::new(AsyncBridge::start(2).expect("start bridge"));
        let callers = (0..4)
            .map(|_| {
                let bridge = Arc::clone(&bridge);
                thread::spawn(move || bridge.shutdown())
            })
            .collect::<Vec<_>>();

        for caller in callers {
            assert_eq!(caller.join().expect("join shutdown caller"), Ok(()));
        }
        assert_eq!(bridge.shutdown(), Ok(()));
        assert_eq!(
            match bridge.submit(async { 1_u8 }) {
                Ok(_) => panic!("stopped bridge must reject new jobs"),
                Err(error) => error,
            },
            AsyncBridgeError::ShuttingDown
        );
    }

    #[test]
    fn shutdown_has_a_hard_deadline_for_non_cooperative_tasks() {
        let grace_timeout = Duration::from_millis(25);
        let runtime_timeout = Duration::from_millis(25);
        let bridge = AsyncBridge::start_with_shutdown_timeouts(1, grace_timeout, runtime_timeout)
            .expect("start bridge");
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);

        let _pending = bridge
            .submit(async move {
                started_tx.send(()).expect("report task started");
                let _ = release_rx.recv();
            })
            .expect("submit non-cooperative task");
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("non-cooperative task started");

        let started_at = Instant::now();
        let result = bridge.shutdown();
        let elapsed = started_at.elapsed();
        release_tx.send(()).expect("release blocked task");

        assert_eq!(
            result,
            Err(AsyncBridgeError::ShutdownTimedOut {
                timeout: grace_timeout + runtime_timeout,
            })
        );
        assert!(
            elapsed < Duration::from_secs(1),
            "shutdown exceeded hard deadline: {elapsed:?}"
        );
    }

    #[test]
    fn spawn_blocking_shutdown_timeout_is_honest_and_sticky_for_all_callers() {
        let grace_timeout = Duration::from_millis(25);
        let runtime_timeout = Duration::from_millis(25);
        let expected_error = AsyncBridgeError::ShutdownTimedOut {
            timeout: grace_timeout + runtime_timeout,
        };
        let bridge = Arc::new(
            AsyncBridge::start_with_shutdown_timeouts(1, grace_timeout, runtime_timeout)
                .expect("start bridge"),
        );
        let (blocking_started_tx, blocking_started_rx) = mpsc::sync_channel(1);
        let (release_blocking_tx, release_blocking_rx) = mpsc::sync_channel(1);

        let pending = bridge
            .submit(async move {
                tokio::task::spawn_blocking(move || {
                    blocking_started_tx
                        .send(())
                        .expect("report blocking task started");
                    let _ = release_blocking_rx.recv();
                })
                .await
                .expect("join blocking task");
            })
            .expect("submit accepted future");
        blocking_started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("blocking task started");

        let started_at = Instant::now();
        let callers = (0..4)
            .map(|_| {
                let bridge = Arc::clone(&bridge);
                thread::spawn(move || bridge.shutdown())
            })
            .collect::<Vec<_>>();
        let shutdown_results = callers
            .into_iter()
            .map(|caller| caller.join().expect("join shutdown caller"))
            .collect::<Vec<_>>();
        let elapsed = started_at.elapsed();
        let repeated_result = bridge.shutdown();

        release_blocking_tx.send(()).expect("release blocking task");
        let pending_result = pending.wait(Duration::from_secs(1));

        assert!(
            shutdown_results
                .iter()
                .all(|result| result == &Err(expected_error.clone())),
            "all shutdown callers must observe the timeout: {shutdown_results:?}"
        );
        assert_eq!(repeated_result, Err(expected_error));
        assert!(
            elapsed < Duration::from_secs(1),
            "shutdown exceeded its hard deadline: {elapsed:?}"
        );
        assert_eq!(pending_result, Err(AsyncBridgeError::ShuttingDown));
    }

    #[test]
    fn shutdown_cancels_accepted_jobs_after_grace_and_joins_worker() {
        let grace_timeout = Duration::from_millis(25);
        let runtime_timeout = Duration::from_millis(25);
        let bridge = Arc::new(
            AsyncBridge::start_with_shutdown_timeouts(2, grace_timeout, runtime_timeout)
                .expect("start bridge"),
        );
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (dropped_tx, dropped_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();
        let (_queued_release_tx, queued_release_rx) = tokio::sync::oneshot::channel::<()>();

        let active_result = bridge
            .submit(async move {
                let _drop_notifier = DropNotifier(Some(dropped_tx));
                started_tx.send(()).expect("report started");
                let _ = release_rx.await;
                1_u8
            })
            .expect("submit active job");
        started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("active job started");
        let second_result = bridge
            .submit(async move {
                let _ = queued_release_rx.await;
                2_u8
            })
            .expect("submit second accepted job");

        let shutdown_bridge = Arc::clone(&bridge);
        let (shutdown_tx, shutdown_rx) = mpsc::sync_channel(1);
        let shutdown_thread = thread::spawn(move || {
            let _ = shutdown_tx.send(shutdown_bridge.shutdown());
        });

        let shutdown_result = shutdown_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("bounded shutdown completed");
        shutdown_thread.join().expect("join shutdown caller");

        assert_eq!(
            shutdown_result,
            Err(AsyncBridgeError::ShutdownTimedOut {
                timeout: grace_timeout + runtime_timeout,
            })
        );
        assert_eq!(dropped_rx.recv_timeout(Duration::from_secs(1)), Ok(()));
        assert_eq!(
            active_result.wait(Duration::from_secs(1)),
            Err(AsyncBridgeError::ShuttingDown)
        );
        assert_eq!(
            second_result.wait(Duration::from_secs(1)),
            Err(AsyncBridgeError::ShuttingDown)
        );
        assert_eq!(
            match bridge.submit(async { 3_u8 }) {
                Ok(_) => panic!("shutdown bridge must reject new jobs"),
                Err(error) => error,
            },
            AsyncBridgeError::ShuttingDown
        );
        drop(release_tx);
    }

    #[test]
    fn submit_is_rejected_without_waiting_for_shutdown_join() {
        let bridge = Arc::new(AsyncBridge::start(1).expect("start bridge"));
        let (blocking_started_tx, blocking_started_rx) = mpsc::sync_channel(1);
        let (release_blocking_tx, release_blocking_rx) = mpsc::sync_channel(1);

        let active_result = bridge
            .submit(async move {
                let _ = tokio::task::spawn_blocking(move || {
                    blocking_started_tx
                        .send(())
                        .expect("report blocking task started");
                    let _ = release_blocking_rx.recv();
                })
                .await;
            })
            .expect("submit active job");
        blocking_started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("blocking task started");

        let shutdown_bridge = Arc::clone(&bridge);
        let (shutdown_result_tx, shutdown_result_rx) = mpsc::sync_channel(1);
        let shutdown_thread = thread::spawn(move || {
            let _ = shutdown_result_tx.send(shutdown_bridge.shutdown());
        });
        let shutdown_started = (0..100).any(|_| {
            let state = bridge
                .lifecycle
                .lock()
                .expect("read bridge lifecycle")
                .state;
            if state == BridgeState::ShuttingDown {
                true
            } else {
                thread::sleep(Duration::from_millis(1));
                false
            }
        });
        assert!(
            shutdown_started,
            "shutdown lifecycle transition was not observed"
        );

        let submit_bridge = Arc::clone(&bridge);
        let (submit_result_tx, submit_result_rx) = mpsc::sync_channel(1);
        let submit_thread = thread::spawn(move || {
            let error = match submit_bridge.submit(async { 2_u8 }) {
                Ok(_) => panic!("shutting down bridge must reject new jobs"),
                Err(error) => error,
            };
            let _ = submit_result_tx.send(error);
        });
        let prompt_submit_result = submit_result_rx
            .recv_timeout(Duration::from_millis(100))
            .ok();

        release_blocking_tx.send(()).expect("release blocking task");
        assert_eq!(
            shutdown_result_rx.recv_timeout(Duration::from_secs(1)),
            Ok(Ok(()))
        );
        shutdown_thread.join().expect("join shutdown caller");
        submit_thread.join().expect("join submit caller");
        assert_eq!(active_result.wait(Duration::from_secs(1)), Ok(()));
        assert_eq!(
            prompt_submit_result,
            Some(AsyncBridgeError::ShuttingDown),
            "submit must not wait for the worker join while shutdown is in progress"
        );
    }

    #[test]
    fn panicking_future_returns_typed_error_without_stopping_worker() {
        let bridge = AsyncBridge::start(1).expect("start bridge");

        assert_eq!(
            bridge
                .submit(async {
                    panic!("future failed");
                    #[allow(unreachable_code)]
                    1_u8
                })
                .expect("submit panicking job")
                .wait(Duration::from_secs(1)),
            Err(AsyncBridgeError::TaskPanicked)
        );
        assert_eq!(
            bridge
                .submit(async { 2_u8 })
                .expect("submit follow-up job")
                .wait(Duration::from_secs(1)),
            Ok(2_u8)
        );
        bridge.shutdown().expect("shutdown bridge");
    }

    #[test]
    fn nested_submission_from_bridge_runtime_completes_without_timeout() {
        let bridge = Arc::new(AsyncBridge::start(4).expect("start bridge"));
        let nested_bridge = Arc::clone(&bridge);

        let result = bridge
            .submit(async move {
                tokio::task::spawn_blocking(move || {
                    nested_bridge
                        .submit(async { 42_u8 })
                        .expect("submit nested job")
                        .wait(Duration::from_millis(100))
                })
                .await
                .expect("join nested blocking caller")
            })
            .expect("submit outer job")
            .wait(Duration::from_secs(1));

        assert_eq!(result, Ok(Ok(42_u8)));
        bridge.shutdown().expect("shutdown bridge");
    }

    #[test]
    fn explicit_timeout_is_applied_by_public_bridge_api() {
        let timeout = Duration::from_millis(25);
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        let release_thread = thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            let _ = release_tx.send(());
        });

        let result = block_on_async_with_timeout(
            async move {
                let _ = release_rx.await;
                1_u8
            },
            timeout,
        );

        release_thread.join().expect("join release thread");
        assert_eq!(result, Err(AsyncBridgeError::TimedOut { timeout }));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn block_on_async_does_not_deadlock_inside_multi_thread_runtime() {
        let value = tokio::task::spawn_blocking(|| block_on_async(async { 42 }))
            .await
            .expect("join");

        assert_eq!(value, Ok(42));
    }
}
