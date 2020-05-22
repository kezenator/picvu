use std::sync::{Arc, Mutex};

pub fn channel() -> (ProgressSender, ProgressReceiver)
{
    let inner = Arc::new(Mutex::new(ProgressInner::new()));

    let sender = ProgressSender { inner: inner.clone() };
    let receiver = ProgressReceiver { inner };

    (sender, receiver)
}

#[derive(Debug, Clone)]
pub struct ProgressState
{
    pub completed_stages: Vec<String>,
    pub current_stage: String,
    pub percentage_complete: f64,
    pub progress_lines: Vec<String>,
    pub remaining_stages: Vec<String>,
    pub complete: bool,
}

#[derive(Clone)]
pub struct ProgressSender
{
    inner: Arc<Mutex<ProgressInner>>,
}

impl ProgressSender
{
    pub fn start_stage(&self, name: String, remaining_stages: Vec<String>)
    {
        let mut data = self.inner.lock().unwrap();

        if data.started_first_stage
        {
            // Mark the previous state as completed

            let prev_stage = data.state.current_stage.clone();
            data.state.completed_stages.push(prev_stage);
        }

        data.started_first_stage = true;
        data.state.current_stage = name;
        data.state.percentage_complete = 0.0;
        data.state.progress_lines.clear();
        data.state.remaining_stages = remaining_stages;
    }

    pub fn set(&self, percentage_complete: f64, progress_lines: Vec<String>)
    {
        let mut data = self.inner.lock().unwrap();

        assert!(data.started_first_stage);

        data.state.percentage_complete = percentage_complete;
        data.state.progress_lines = progress_lines;
    }
}

#[derive(Clone)]
pub struct ProgressReceiver
{
    inner: Arc<Mutex<ProgressInner>>,
}

impl ProgressReceiver
{
    pub fn get_state(&self) -> ProgressState
    {
        self.inner.lock().unwrap().state.clone()
    }
}

struct ProgressInner
{
    started_first_stage: bool,
    state: ProgressState,
}

impl ProgressInner
{
    pub fn new() -> Self
    {
        let started_first_stage = false;
        let state = ProgressState
        {
            completed_stages: Vec::new(),
            current_stage: "Starting...".to_owned(),
            percentage_complete: 0.0,
            progress_lines: Vec::new(),
            remaining_stages: Vec::new(),
            complete: false,
        };

        ProgressInner { started_first_stage, state }
    }
}