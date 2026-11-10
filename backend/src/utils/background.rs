//! Background task helpers.
//!
//! Sending an email is an outbound network call to a third-party relay. Doing
//! it inline in the request path means a slow relay holds an Actix worker and
//! slows the client's response — an amplifier a registration bot can exploit
//! for free. Dispatching the send to the runtime keeps the response latency
//! independent of the mail relay.

use std::future::Future;

/// Run a task on the Actix runtime without awaiting it in the request path.
///
/// The task owns its data (it must be `Send + 'static`); failures are the
/// task's own responsibility — log them there, because nobody awaits the
/// result.
pub fn spawn_background<F>(task: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    actix_web::rt::spawn(task);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[actix_web::test]
    async fn spawned_task_runs_on_the_runtime() {
        let finished = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&finished);

        spawn_background(async move {
            flag.store(true, Ordering::SeqCst);
        });

        // The task is scheduled, not awaited: give the runtime a few ticks.
        for _ in 0..100 {
            if finished.load(Ordering::SeqCst) {
                break;
            }
            actix_web::rt::task::yield_now().await;
        }

        assert!(
            finished.load(Ordering::SeqCst),
            "the spawned task must execute on the Actix runtime"
        );
    }

    #[actix_web::test]
    async fn spawning_does_not_panic_for_a_failing_task() {
        // The task's outcome is its own business — the dispatcher never
        // propagates it, so a failed send cannot break the request that
        // triggered it.
        spawn_background(async {
            panic!("task failure stays inside the task");
        });

        actix_web::rt::task::yield_now().await;
    }
}
