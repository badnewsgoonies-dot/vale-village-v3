//! Screenshot helpers for automated visual capture.

use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};

/// Request a primary-window screenshot after a small frame delay.
#[derive(Resource, Clone, Debug)]
pub struct ScreenshotRequest {
    pub output_path: String,
    pub frames_to_wait: u32,
}

/// Lightweight wrapper so UI callers can opt into screenshot capture.
pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, capture_screenshot);
    }
}

/// Capture a primary-window screenshot once the requested delay elapses.
pub fn capture_screenshot(
    mut commands: Commands,
    request: Option<ResMut<ScreenshotRequest>>,
) {
    let Some(mut request) = request else {
        return;
    };

    if request.frames_to_wait > 0 {
        request.frames_to_wait -= 1;
        return;
    }

    let output_path = request.output_path.clone();
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(output_path));
    commands.remove_resource::<ScreenshotRequest>();
}
