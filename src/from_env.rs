use bevy::{ecs::schedule::ReportExecutionOrderAmbiguities, prelude::*};

pub struct ExecutionOrderAmbiguitiesPlugin;

impl Plugin for ExecutionOrderAmbiguitiesPlugin {
    fn build(&self, app: &mut App) {
        if get_should_report_execution_order_ambiguities().unwrap_or_default() {
            app.insert_resource(ReportExecutionOrderAmbiguities);
        }
    }
}

pub trait FromEnv: Default {
    fn from_env() -> Self {
        Self::with_env_overrides(Self::default())
    }

    fn with_env_overrides(self) -> Self;
}

impl FromEnv for WindowDescriptor {
    fn with_env_overrides(mut self) -> Self {
        if let Some(width) = get_window_width() {
            self.width = width;
        }

        if let Some(height) = get_window_height() {
            self.height = height;
        }

        if let Some(decorations) = get_window_decorations() {
            self.decorations = decorations;
        }

        if let Some(position) = get_window_absolute_position().or_else(get_window_monitor) {
            self.position = position;
        }

        self
    }
}

fn get_window_width() -> Option<f32> {
    std::env::var("WINDOW_WIDTH").ok()?.parse().ok()
}

fn get_window_height() -> Option<f32> {
    std::env::var("WINDOW_HEIGHT").ok()?.parse().ok()
}

fn get_window_decorations() -> Option<bool> {
    std::env::var("WINDOW_DECORATIONS").ok()?.parse().ok()
}

fn get_window_absolute_position() -> Option<WindowPosition> {
    let window_x = std::env::var("WINDOW_X").ok()?.parse().ok()?;
    let window_y = std::env::var("WINDOW_Y").ok()?.parse().ok()?;
    Some(WindowPosition::At(Vec2::new(window_x, window_y)))
}

fn get_window_monitor() -> Option<WindowPosition> {
    let window_monitor = std::env::var("WINDOW_MONITOR").ok()?;

    let window_monitor = match window_monitor.to_lowercase().as_str() {
        "current" => MonitorSelection::Current,
        "primary" => MonitorSelection::Primary,
        x => MonitorSelection::Number(x.parse().ok()?),
    };

    Some(WindowPosition::Centered(window_monitor))
}

fn get_should_report_execution_order_ambiguities() -> Option<bool> {
    std::env::var("REPORT_AMBIG").ok()?.parse().ok()
}
