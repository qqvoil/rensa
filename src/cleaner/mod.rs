pub mod runner;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CleanEvent {
    StepStart { id: String, title: String },
    LogLine(String),
    StepDone { id: String, success: bool, message: String },
    Finished { total_steps: usize, successful_steps: usize },
}
