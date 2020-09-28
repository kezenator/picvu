use std::future::Future;
use std::sync::{Arc, Mutex};

pub mod export;
pub mod import;
pub mod progress;
pub mod sync;
pub mod tags;

pub trait BulkOperation
{
    type Error: std::fmt::Debug;
    type Future: Future<Output = Result<(), Self::Error>> + 'static;

    fn name(&self) -> String;
    fn start(self, sender: progress::ProgressSender) -> Self::Future;
}

pub struct BulkQueue
{
    inner: Arc<Mutex<BulkQueueInner>>,
}

impl BulkQueue
{
    pub fn new() -> Self
    {
        BulkQueue
        {
            inner: Arc::new(Mutex::new(BulkQueueInner::None)),
        }
    }

    pub fn enqueue(&mut self, op: impl BulkOperation)
    {
        let inner_cloned = self.inner.clone();

        let mut inner = self.inner.lock().unwrap();

        assert!(if let BulkQueueInner::None = *inner { true } else { false });

        let (tx, rx) = progress::channel();

        *inner = BulkQueueInner::InProgress(rx.clone());

        let future = op.start(tx.clone());

        actix_rt::spawn(async move
            {
                match future.await
                {
                    Ok(_) => {},
                    Err(err) =>
                    {
                        tx.start_stage("Failed".to_owned(), vec![]);

                        let lines = format!("{:#?}", err)
                            .split("\n")
                            .map(|s| s.to_owned())
                            .collect();

                        tx.set(100.0, lines);
                    },
                }

                // Mark that we're completed
                {
                    let mut state = rx.get_state();
                    state.complete = true;
                    *inner_cloned.lock().unwrap() = BulkQueueInner::Completed(state);
                }
            });
    }

    pub fn remove_completed(&self)
    {
        let mut inner = self.inner.lock().unwrap();

        if let BulkQueueInner::Completed(_) = *inner
        {
            *inner = BulkQueueInner::None;
        }
    }

    pub fn get_current_progress(&self) -> Option<progress::ProgressState>
    {
        let inner = self.inner.lock().unwrap();

        match &*inner
        {
            BulkQueueInner::None => None,
            BulkQueueInner::InProgress(rx) => Some(rx.get_state()),
            BulkQueueInner::Completed(state) => Some(state.clone()),
        }
    }
}

enum BulkQueueInner
{
    None,
    InProgress(progress::ProgressReceiver),
    Completed(progress::ProgressState),
}