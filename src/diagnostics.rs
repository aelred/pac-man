use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorParams, WorldInspectorPlugin};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(WorldInspectorPlugin::new())
            .insert_resource(WorldInspectorParams {
                enabled: false,
                ..default()
            })
            .add_system(toggle_inspector);
    }
}

fn toggle_inspector(
    mut inspector_params: ResMut<WorldInspectorParams>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Grave) {
        inspector_params.enabled = !inspector_params.enabled;
    }
}
