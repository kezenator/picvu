use std::future::Future;
use actix_web::HttpResponse;
use std::sync::{Arc, Mutex};
use horrorshow::html;

use crate::view;

pub mod import;
pub mod progress;

pub trait BulkOperation
{
    type Error;
    type Future: Future<Output = Result<(), Self::Error>> + 'static;

    fn name(&self) -> String;
    fn start(self, sender: progress::ProgressSender) -> Self::Future;
}

pub struct BulkQueue
{
    inner: Arc<Mutex<Option<progress::ProgressReceiver>>>,
}

impl BulkQueue
{
    pub fn new() -> Self
    {
        BulkQueue
        {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub fn enqueue(&mut self, op: impl BulkOperation)
    {
        let inner_cloned = self.inner.clone();

        let mut opt_rx = self.inner.lock().unwrap();

        assert!{opt_rx.is_none()};

        let (tx, rx) = progress::channel();

        *opt_rx = Some(rx);

        let future = op.start(tx.clone());

        actix_rt::spawn(async move
            {
                // TODO - better error handling....
                match future.await
                {
                    Ok(_) => {},
                    Err(_) => { tx.set(html!{ : "Completed with errors!" }); },
                }

                // Mark that we're finished
                *inner_cloned.lock().unwrap() = None;
            });
    }

    pub fn is_op_in_progress(&self) -> bool
    {
        let opt_rx = self.inner.lock().unwrap();

        return opt_rx.is_some();
    }

    pub fn render(&self) -> HttpResponse
    {
        let opt_rx = self.inner.lock().unwrap();

        let title = "Background Tasks".to_owned();

        let contents = match &*opt_rx
        {
            None => "No background tasks in operation.".to_owned(),
            Some(rx) => rx.get_raw_html(),
        };

        view::doc::ok(view::page::Page{ title, contents })
    }
}