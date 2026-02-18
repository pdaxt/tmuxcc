use std::sync::{Arc, Mutex};
use crate::state::StateManager;
use crate::pty::PtyManager;

pub struct App {
    pub state: Arc<StateManager>,
    pub pty: Arc<Mutex<PtyManager>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: Arc::new(StateManager::new()),
            pty: Arc::new(Mutex::new(PtyManager::new())),
        }
    }
}
