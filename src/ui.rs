use crate::grid::{GridBundle, GridLocation, Layer};
use crate::level::GRID;
use crate::score::{HighScore, Score, UpdateHighScore, UpdateScore};
use crate::text::{Align, Text, TextBundle, TextPlugin};
use crate::HEIGHT_TILES;
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TextPlugin)
            .add_startup_system(setup_ui)
            .add_system(update_score_display.after(UpdateScore))
            .add_system(update_high_score_display.after(UpdateHighScore));
    }
}

#[derive(Component)]
struct ScoreDisplay;

#[derive(Component)]
struct HighScoreDisplay;

fn setup_ui(mut commands: Commands) {
    commands
        .spawn_bundle(SpatialBundle::default())
        .insert(Name::new("UI"))
        .with_children(|builder| {
            builder
                .spawn_bundle(TextBundle {
                    text: Text {
                        string: "1UP   HIGH SCORE".to_string(),
                        ..default()
                    },
                    ..default()
                })
                .insert(Name::new("Static Text"))
                .insert_bundle(GridBundle {
                    grid: GRID,
                    location: GridLocation {
                        x: 3,
                        y: HEIGHT_TILES as isize - 1,
                    },
                    layer: Layer::UI,
                    ..default()
                });

            builder
                .spawn_bundle(TextBundle {
                    text: Text {
                        align: Align::Right,
                        ..default()
                    },
                    ..default()
                })
                .insert_bundle(GridBundle {
                    grid: GRID,
                    location: GridLocation {
                        x: 6,
                        y: HEIGHT_TILES as isize - 2,
                    },
                    layer: Layer::UI,
                    ..default()
                })
                .insert(Name::new("Score"))
                .insert(ScoreDisplay);

            builder
                .spawn_bundle(TextBundle {
                    text: Text {
                        align: Align::Right,
                        ..default()
                    },
                    ..default()
                })
                .insert_bundle(GridBundle {
                    grid: GRID,
                    location: GridLocation {
                        x: 16,
                        y: HEIGHT_TILES as isize - 2,
                    },
                    layer: Layer::UI,
                    ..default()
                })
                .insert(Name::new("High Score"))
                .insert(HighScoreDisplay);
        });
}

fn update_score_display(score: ResMut<Score>, mut query: Query<&mut Text, With<ScoreDisplay>>) {
    if score.is_changed() {
        for mut text in &mut query {
            text.string = score.to_string();
        }
    }
}

fn update_high_score_display(
    high_score: ResMut<HighScore>,
    mut query: Query<&mut Text, With<HighScoreDisplay>>,
) {
    if high_score.is_changed() {
        for mut text in &mut query {
            text.string = high_score.to_string();
        }
    }
}
