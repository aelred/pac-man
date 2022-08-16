mod diagnostics;
mod food;
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
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::camera::{DepthCalculation, ScalingMode};
use bevy::render::texture::ImageSettings;
use layout::LayoutPlugin;
use level::GRID_SIZE;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: get_window_width().unwrap_or(WIDTH * SCALE),
            height: get_window_height().unwrap_or(HEIGHT * SCALE),
            position: get_window_position().unwrap_or(WindowPosition::Automatic),
            decorations: get_window_decorations().unwrap_or(true),
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup_camera)
        .add_system(exit_game)
        .add_plugins(DefaultPlugins)
        .add_plugin(GridPlugin)
        .add_plugin(UIPlugin)
        .add_plugin(InspectorPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(FoodPlugin)
        .add_plugin(ScorePlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(LayoutPlugin)
        .run();
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

fn get_window_position() -> Option<WindowPosition> {
    let window_x = std::env::var("WINDOW_X").ok()?.parse().ok()?;
    let window_y = std::env::var("WINDOW_Y").ok()?.parse().ok()?;
    Some(WindowPosition::At(Vec2::new(window_x, window_y)))
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
