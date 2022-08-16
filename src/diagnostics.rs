use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

use crate::grid::GridLocation;
use crate::level::GRID;
use crate::player::PlayerDied;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(DebugLinesPlugin::default())
            .init_resource::<DebugMode>()
            .insert_resource(WorldInspectorParams {
                enabled: false,
                ..default()
            })
            .add_system(toggle_debug_mode)
            .add_system(toggle_inspector)
            .add_system(trigger_death)
            .add_system(draw_grid);
    }
}

#[derive(Default)]
struct DebugMode(bool);

fn toggle_debug_mode(mut debug_mode: ResMut<DebugMode>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Grave) {
        debug_mode.0 = !debug_mode.0;
    }
}

fn toggle_inspector(
    debug_mode: Res<DebugMode>,
    mut inspector_params: ResMut<WorldInspectorParams>,
) {
    if debug_mode.is_changed() {
        inspector_params.enabled = debug_mode.0;
    }
}

fn trigger_death(keyboard_input: Res<Input<KeyCode>>, mut death_events: EventWriter<PlayerDied>) {
    if keyboard_input.just_pressed(KeyCode::Period) {
        death_events.send(PlayerDied);
    }
}

fn draw_grid(debug_mode: Res<DebugMode>, mut lines: ResMut<DebugLines>) {
    let offset = GRID.size / 2.0;

    if debug_mode.0 {
        for x in -100..100 {
            let start = (GRID.to_vec2(GridLocation { x, y: -100 }) + offset).extend(0.0);
            let end = (GRID.to_vec2(GridLocation { x, y: 100 }) + offset).extend(0.0);
            lines.line(start, end, 0.0);
        }
        for y in -100..100 {
            let start = (GRID.to_vec2(GridLocation { x: -100, y }) + offset).extend(0.0);
            let end = (GRID.to_vec2(GridLocation { x: 100, y }) + offset).extend(0.0);
            lines.line(start, end, 0.0);
        }
    }
}
