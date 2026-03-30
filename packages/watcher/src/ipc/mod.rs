use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

mod server;
mod client;

#[derive(Debug, Clone)]
pub(crate) struct ControlState {
    pub(crate) paused: bool,
    pub(crate) started_at: u64,
    pub(crate) last_capture_at: Option<u64>,
    pub(crate) last_error: Option<String>,
    pub(crate) db_path: String,
    pub(crate) image_dir: String,
}

pub(crate) type SharedControlState = Arc<Mutex<ControlState>>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ControlResponse {
    pub(crate) ok: bool,
    pub(crate) message: String,
    pub(crate) paused: bool,
    pub(crate) started_at: u64,
    pub(crate) last_capture_at: Option<u64>,
    pub(crate) last_error: Option<String>,
    pub(crate) db_path: String,
    pub(crate) image_dir: String,
}

pub(crate) use server::{start_control_server, new_control_state};
pub(crate) use client::{send_control_command, print_control_response};
