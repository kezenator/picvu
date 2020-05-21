use std::sync::{Arc, Mutex};
use horrorshow::prelude::*;

pub fn channel() -> (ProgressSender, ProgressReceiver)
{
    let inner = Arc::new(Mutex::new(ProgressInner::new()));

    let sender = ProgressSender { inner: inner.clone() };
    let receiver = ProgressReceiver { inner };

    (sender, receiver)
}

#[derive(Clone)]
pub struct ProgressSender
{
    inner: Arc<Mutex<ProgressInner>>,
}

impl ProgressSender
{
    pub fn set<F>(&self, html: FnRenderer<F>)
        where F: FnOnce(&mut TemplateBuffer)
    {
        let mut data = self.inner.lock().unwrap();

        data.raw_value = html.into_string().unwrap();
    }
}

#[derive(Clone)]
pub struct ProgressReceiver
{
    inner: Arc<Mutex<ProgressInner>>,
}

impl ProgressReceiver
{
    pub fn get_raw_html(&self) -> String
    {
        self.inner.lock().unwrap().raw_value.clone()
    }
}

struct ProgressInner
{
    raw_value: String,
}

impl ProgressInner
{
    pub fn new() -> Self
    {
        ProgressInner { raw_value: "Starting...".to_owned() }
    }
}