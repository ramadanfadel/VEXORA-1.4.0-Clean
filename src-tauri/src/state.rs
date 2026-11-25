//! Application state shared by all Tauri commands.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

use crate::registry::Issue;

pub struct AppState {
    pub settings: Mutex<crate::settings::Settings>,
    pub registry_issues: Mutex<Vec<Issue>>,
    pub cancel: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            settings: Mutex::new(crate::settings::Settings::load()),
            registry_issues: Mutex::new(Vec::new()),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}
