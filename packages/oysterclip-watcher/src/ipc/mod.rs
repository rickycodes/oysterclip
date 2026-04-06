use std::sync::{Arc, Mutex};

use common::ControlResponse;

mod client;
mod server;

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

pub(crate) use client::{print_control_response, send_control_command};
pub(crate) use server::{new_control_state, start_control_server};
