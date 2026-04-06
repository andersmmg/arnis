#[cfg(feature = "gui")]
use crate::telemetry::{send_log, LogLevel};
use once_cell::sync::OnceCell;
use serde_json::json;
use std::time::Instant;
use tauri::{Emitter, WebviewWindow};

pub static MAIN_WINDOW: OnceCell<WebviewWindow> = OnceCell::new();
static START_TIME: OnceCell<Instant> = OnceCell::new();

pub fn set_main_window(window: WebviewWindow) {
    MAIN_WINDOW.set(window).ok();
}

pub fn get_main_window() -> Option<&'static WebviewWindow> {
    MAIN_WINDOW.get()
}

fn start_timer_if_needed() {
    if START_TIME.get().is_none() {
        START_TIME.set(Instant::now()).ok();
    }
}

fn get_elapsed_secs() -> f64 {
    START_TIME
        .get()
        .map(|t| t.elapsed().as_secs_f64())
        .unwrap_or(0.0)
}

fn format_eta(seconds: f64) -> String {
    if seconds >= 60.0 {
        let mins = (seconds / 60.0).round() as i32;
        format!("~{}m", mins)
    } else {
        format!("~{}s", seconds.round() as i32)
    }
}

/// This function checks if the program is running with a GUI window.
/// Returns `true` if a GUI window is initialized, `false` otherwise.
pub fn is_running_with_gui() -> bool {
    get_main_window().is_some()
}

/// This code manages a multi-step process with a progress bar indicating the overall completion.
/// The progress updates are mapped to specific steps in the pipeline:
///
/// [1/7] Fetching data... - Starts at: 0% / Completes at: 5%
/// [2/7] Parsing data... - Starts at: 5% / Completes at: 15%
/// [3/7] Fetching elevation... - Starts at: 15% / Completes at: 20%
/// [4/7] Transforming map... - Starts at: 20% / Completes at: 25%
/// [5/7] Processing terrain... - Starts at: 25% / Completes at: 70%
/// [6/7] Generating ground... - Starts at: 70% / Completes at: 90%
/// [7/7] Saving world... - Starts at: 90% / Completes at: 100%
///
/// The function `emit_gui_progress_update` is used to send real-time progress updates to the UI.
pub fn emit_gui_progress_update(
    progress: f64,
    message: &str,
    step: Option<&str>,
    force_estimate: Option<&str>,
) {
    start_timer_if_needed();

    let estimate_str = if let Some(est) = force_estimate {
        est.to_string()
    } else if progress > 0.0 && progress < 100.0 {
        let elapsed = get_elapsed_secs();
        if elapsed > 0.5 {
            let total_estimated = (elapsed / progress) * 100.0;
            let remaining = total_estimated - elapsed;
            if remaining > 0.0 {
                format_eta(remaining)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    if let Some(window) = get_main_window() {
        let payload = json!({
            "progress": progress,
            "message": message,
            "step": step.unwrap_or(""),
            "estimate": estimate_str
        });

        if let Err(e) = window.emit("progress-update", payload) {
            let error_msg = format!("Failed to emit progress event: {}", e);
            eprintln!("{}", error_msg);
            #[cfg(feature = "gui")]
            send_log(LogLevel::Warning, &error_msg);
        }
    }
}

pub fn emit_gui_error(message: &str) {
    let truncated_message = if message.len() > 35 {
        &message[..35]
    } else {
        message
    };
    emit_gui_progress_update(0.0, &format!("Error! {truncated_message}"), None, None);
}

/// Emits an event when the world map preview is ready
pub fn emit_map_preview_ready() {
    if let Some(window) = get_main_window() {
        if let Err(e) = window.emit("map-preview-ready", ()) {
            eprintln!("Failed to emit map-preview-ready event: {}", e);
        }
    }
}

/// Emits an event to reveal a file or folder in the system file explorer
pub fn emit_show_in_folder(path: &str) {
    if let Some(window) = get_main_window() {
        if let Err(e) = window.emit("show-in-folder", path) {
            eprintln!("Failed to emit show-in-folder event: {}", e);
        }
    }
}
