mod diagnostics;
mod food;
mod game_over;
mod grid;
mod layout;
mod level;
mod movement;
mod player;
mod score;
mod text;
mod ui;

use crate::diagnostics::InspectorPlugin;
use crate::food::FoodPlugin;
use crate::grid::GridPlugin;
use crate::level::{LevelPlugin, HEIGHT, HEIGHT_TILES, SCALE, WIDTH, WIDTH_TILES};
use crate::movement::MovementPlugin;
use crate::player::PlayerPlugin;
use crate::score::ScorePlugin;
use crate::ui::UIPlugin;
use bevy::prelude::*;
use bevy::render::camera::{DepthCalculation, ScalingMode};
use bevy::render::texture::ImageSettings;
use game_over::GameOverPlugin;
use layout::LayoutPlugin;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: WIDTH * SCALE,
            height: HEIGHT * SCALE,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup_camera)
        .add_plugins(DefaultPlugins)
        .add_plugin(GridPlugin)
        .add_plugin(UIPlugin)
        .add_plugin(InspectorPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(FoodPlugin)
        .add_plugin(ScorePlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(GameOverPlugin)
        .add_plugin(LayoutPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Auto {
                min_width: WIDTH,
                min_height: HEIGHT,
            },
            far: 1000.0,
            depth_calculation: DepthCalculation::ZDifference,
            ..default()
        },
        ..default()
    });
}