//! One algorithm, two drivers.
//!
//! Writing and reading a document are pure computation, each written once as
//! a [`Steps`] job: a value that does a bounded amount of work per
//! [`Steps::step`] call and keeps its state between calls. The sync
//! functions drive a job to the end with [`run`]. The async functions of
//! [`crate::tokio`] drive the same job with [`run_async`], which gives the
//! task back to the scheduler between steps when its cooperative budget is
//! spent. No thread is involved in either.

use crate::error::Error;

/// A computation cut into bounded steps.
pub(crate) trait Steps {
    /// What the job produces when its steps are done.
    type Output;

    /// Do one bounded unit of work. `Ok(None)` means more steps remain;
    /// `Ok(Some(output))` ends the job. A job is not stepped again after it
    /// ended or failed.
    fn step(&mut self) -> Result<Option<Self::Output>, Error>;
}

/// Run `job` to the end with no yield point.
pub(crate) fn run<S: Steps>(mut job: S) -> Result<S::Output, Error> {
    loop {
        if let Some(output) = job.step()? {
            return Ok(output);
        }
    }
}

/// Run `job` to the end, calling `tokio::task::consume_budget` between
/// steps.
///
/// The call yields to the scheduler only when the task's cooperative
/// budget is spent, so a short document never yields. Outside a tokio
/// runtime there is no budget and it never yields.
#[cfg(feature = "tokio")]
pub(crate) async fn run_async<S: Steps>(mut job: S) -> Result<S::Output, Error> {
    loop {
        if let Some(output) = job.step()? {
            return Ok(output);
        }
        tokio::task::consume_budget().await;
    }
}

#[cfg(test)]
mod tests {
    use super::{Steps, run};
    use crate::error::{Error, ErrorKind};

    /// Counts up to `end`, one number per step, and fails at `fail_at`.
    pub(super) struct Count {
        next: u32,
        end: u32,
        fail_at: u32,
        seen: Vec<u32>,
    }

    impl Steps for Count {
        type Output = Vec<u32>;

        fn step(&mut self) -> Result<Option<Vec<u32>>, Error> {
            if self.next == self.fail_at {
                return Err(Error::body("the failing step"));
            }
            if self.next == self.end {
                return Ok(Some(std::mem::take(&mut self.seen)));
            }
            self.seen.push(self.next);
            self.next += 1;
            Ok(None)
        }
    }

    pub(super) fn count(end: u32) -> Count {
        Count {
            next: 0,
            end,
            fail_at: u32::MAX,
            seen: Vec::new(),
        }
    }

    #[test]
    fn a_job_runs_every_step_in_order() {
        assert_eq!(run(count(3)).expect("run"), [0, 1, 2]);
        assert!(run(count(0)).expect("run").is_empty());
    }

    #[test]
    fn a_failed_step_ends_the_run_with_its_error() {
        let err = run(Count {
            fail_at: 99,
            ..count(200)
        })
        .expect_err("step 99 fails");
        assert_eq!(err.kind(), ErrorKind::Body);
    }

    #[cfg(feature = "tokio")]
    mod with_tokio {
        use std::future::Future;
        use std::pin::pin;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::task::{Context, Poll, Waker};

        use super::super::{Steps, run, run_async};
        use super::count;
        use crate::error::Error;

        #[test]
        fn the_async_driver_gives_the_sync_result_and_does_not_yield_outside_a_runtime() {
            let expected = run(count(10_000)).expect("run");
            let mut future = pin!(run_async(count(10_000)));
            let Poll::Ready(result) = future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
            else {
                panic!("no runtime, no budget, so no yield");
            };
            assert_eq!(result.expect("run"), expected);
        }

        /// Notes whether a flag was already set when each step ran.
        struct Watch {
            flag: Arc<AtomicBool>,
            left: u32,
            saw_flag: bool,
        }

        impl Steps for Watch {
            type Output = bool;

            fn step(&mut self) -> Result<Option<bool>, Error> {
                self.saw_flag |= self.flag.load(Ordering::SeqCst);
                if self.left == 0 {
                    return Ok(Some(self.saw_flag));
                }
                self.left -= 1;
                Ok(None)
            }
        }

        #[test]
        fn a_long_job_lets_another_task_run_on_a_single_thread() {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("runtime");
            let saw_flag = runtime.block_on(async {
                let flag = Arc::new(AtomicBool::new(false));
                let setter = Arc::clone(&flag);
                let other = tokio::spawn(async move { setter.store(true, Ordering::SeqCst) });
                let saw_flag = run_async(Watch {
                    flag,
                    left: 10_000,
                    saw_flag: false,
                })
                .await
                .expect("run");
                other.await.expect("the other task");
                saw_flag
            });
            assert!(
                saw_flag,
                "the job never gave the thread to the other task before it ended"
            );
        }
    }
}
