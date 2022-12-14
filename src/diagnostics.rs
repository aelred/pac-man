use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorParams, WorldInspectorPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

use crate::actor::ghost::{Blinky, Clyde, Ghost, Inky, Pinky, SetTarget, Target};
use crate::actor::mode::{FrightenedMode, SetMode};
use crate::actor::movement::{Dir, NextDir, SetDir, SetNextDir};
use crate::actor::player::{PlayerDeath, PlayerDied};
use crate::food::{Eat, Food, WriteEatEvent};
use crate::from_env::{ExecutionOrderAmbiguitiesPlugin, FromEnv};
use crate::grid::{Grid, GridLocation, SetGridLocation};
use crate::level::GRID;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LogDiagnosticsPlugin::from_env())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(DebugLinesPlugin::default())
            .add_plugin(ExecutionOrderAmbiguitiesPlugin)
            .init_resource::<DebugMode>()
            .insert_resource(WorldInspectorParams {
                enabled: false,
                ..default()
            })
            .add_system(toggle_debug_mode)
            .add_system_set(
                SystemSet::new()
                    .after(toggle_debug_mode)
                    .label(DebugSystem)
                    // We don't care much about the ordering of debug stuff
                    .with_system(toggle_inspector.ambiguous_with(DebugSystem))
                    .with_system(
                        trigger_eat_ghost
                            .label(WriteEatEvent)
                            .ambiguous_with(WriteEatEvent)
                            .ambiguous_with(DebugSystem),
                    )
                    .with_system(draw_grid.ambiguous_with(DebugSystem))
                    .with_system(draw_dir.after(SetDir).ambiguous_with(DebugSystem))
                    .with_system(
                        draw_next_dir
                            .after(SetGridLocation)
                            .after(SetDir)
                            .after(SetNextDir)
                            .ambiguous_with(DebugSystem),
                    )
                    .with_system(
                        draw_target
                            .after(SetTarget)
                            .after(draw_grid)
                            .ambiguous_with(DebugSystem),
                    ),
            )
            .add_system(trigger_death.label(PlayerDeath).ambiguous_with(PlayerDeath))
            .add_system(trigger_frightened.before(SetMode))
            .register_inspectable::<NextDir>()
            .register_inspectable::<Dir>()
            .register_inspectable::<GridLocation>();
    }
}

#[derive(Resource, Default)]
struct DebugMode(bool);

#[derive(SystemLabel)]
struct DebugSystem;

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

fn trigger_frightened(keyboard_input: Res<Input<KeyCode>>, mut mode: ResMut<FrightenedMode>) {
    if keyboard_input.just_pressed(KeyCode::Comma) {
        *mode = FrightenedMode::Enabled;
    }
}

fn trigger_eat_ghost(
    debug_mode: Res<DebugMode>,
    keyboard_input: Res<Input<KeyCode>>,
    blinky: Query<Entity, (With<Blinky>, With<Food>)>,
    pinky: Query<Entity, (With<Pinky>, With<Food>)>,
    inky: Query<Entity, (With<Inky>, With<Food>)>,
    clyde: Query<Entity, (With<Clyde>, With<Food>)>,
    mut eat_events: EventWriter<Eat>,
) {
    if !debug_mode.0 {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::B) {
        for ghost in &blinky {
            eat_events.send(Eat(ghost));
        }
    }

    if keyboard_input.just_pressed(KeyCode::P) {
        for ghost in &pinky {
            eat_events.send(Eat(ghost));
        }
    }

    if keyboard_input.just_pressed(KeyCode::I) {
        for ghost in &inky {
            eat_events.send(Eat(ghost));
        }
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        for ghost in &clyde {
            eat_events.send(Eat(ghost));
        }
    }
}

fn draw_grid(debug_mode: Res<DebugMode>, mut lines: ResMut<DebugLines>) {
    if !debug_mode.0 {
        return;
    }

    let offset = GRID.size / 2.0;

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

fn draw_target(
    debug_mode: Res<DebugMode>,
    mut lines: ResMut<DebugLines>,
    query: Query<(&Ghost, &Grid, &Target)>,
) {
    if !debug_mode.0 {
        return;
    }

    for (ghost, grid, target) in &query {
        let target = grid.to_vec2(**target).extend(0.0);
        let x = (Vec2::X * grid.size / 4.0).extend(0.0);
        let y = (Vec2::Y * grid.size / 4.0).extend(0.0);
        lines.line_colored(target - x - y, target + x + y, 0.0, ghost.color());
        lines.line_colored(target - x + y, target + x - y, 0.0, ghost.color());
    }
}

fn draw_dir(
    debug_mode: Res<DebugMode>,
    mut lines: ResMut<DebugLines>,
    query: Query<(&Dir, &Grid, &GridLocation)>,
) {
    if !debug_mode.0 {
        return;
    }

    for (dir, grid, location) in &query {
        let next_location = location.shift(*dir);
        let start = grid.to_vec2(*location).extend(0.0);
        let end = grid.to_vec2(next_location).extend(0.0);
        lines.line(start, end, 0.0);
    }
}

fn draw_next_dir(
    debug_mode: Res<DebugMode>,
    mut lines: ResMut<DebugLines>,
    query: Query<(&Dir, &NextDir, &Grid, &GridLocation)>,
) {
    if !debug_mode.0 {
        return;
    }

    for (dir, next_dir, grid, location) in &query {
        let next_dir = match **next_dir {
            Some(n) => n,
            None => continue,
        };

        let next_location = location.shift(*dir);
        let final_location = next_location.shift(next_dir);
        let start = grid.to_vec2(next_location).extend(0.0);
        let end = grid.to_vec2(final_location).extend(0.0);
        lines.line(start, end, 0.0);
    }
}
