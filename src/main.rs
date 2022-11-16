mod actor;
mod diagnostics;
mod food;
mod from_env;
mod grid;
mod layout;
mod level;
mod score;
mod text;
mod ui;

use crate::actor::movement::MovementPlugin;
use crate::actor::player::PlayerPlugin;
use crate::diagnostics::InspectorPlugin;
use crate::food::FoodPlugin;
use crate::grid::GridPlugin;
use crate::level::{LevelPlugin, HEIGHT, HEIGHT_TILES, SCALE, WIDTH};
use crate::score::ScorePlugin;
use crate::ui::UIPlugin;
use actor::ghost::GhostPlugin;
use actor::mode::ModePlugin;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use from_env::FromEnv;
use layout::LayoutPlugin;
use level::GRID_SIZE;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup_camera)
        .add_system(exit_game)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: WIDTH * SCALE,
                        height: HEIGHT * SCALE,
                        ..default()
                    }
                    .with_env_overrides(),
                    ..default()
                }),
        )
        .add_plugin(GridPlugin)
        .add_plugin(UIPlugin)
        .add_plugin(InspectorPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(FoodPlugin)
        .add_plugin(ScorePlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(LayoutPlugin)
        .add_plugin(GhostPlugin)
        .add_plugin(ModePlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Auto {
                min_width: WIDTH,
                min_height: HEIGHT,
            },
            far: 1000.0,
            ..default()
        },
        transform: Transform::from_xyz(
            (WIDTH - GRID_SIZE) / 2.0,
            (HEIGHT - GRID_SIZE) / 2.0,
            999.9,
        ),
        ..default()
    });
}

fn exit_game(input: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}
