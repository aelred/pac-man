use std::cmp::Ordering;

use crate::grid::{Grid, GridBundle, GridLocation, Layer};
use crate::level::{GRID, GRID_SIZE};
use crate::player::Lives;
use crate::score::{HighScore, Score, UpdateHighScore, UpdateScore};
use crate::text::{Align, Text, TextBundle, TextPlugin};
use crate::HEIGHT_TILES;
use bevy::prelude::*;
use bevy::sprite::{Anchor, Rect};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TextPlugin)
            .add_startup_system(setup_ui)
            .add_system(update_score_display.after(UpdateScore))
            .add_system(update_high_score_display.after(UpdateHighScore))
            .add_system(update_lives_display)
            .init_resource::<UIAssets>();
    }
}

#[derive(Component)]
struct ScoreDisplay;

#[derive(Component)]
struct HighScoreDisplay;

#[derive(Component)]
struct LivesDisplay;

#[derive(Deref, DerefMut)]
struct UIAssets(Handle<TextureAtlas>);

impl FromWorld for UIAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server: &AssetServer = world.resource();
        let sheet = asset_server.load("sprite_sheet.png");

        let mut texture_atlases: Mut<Assets<TextureAtlas>> = world.resource_mut();

        let mut atlas = TextureAtlas::new_empty(sheet, Vec2::new(680.0, 248.0));

        atlas.add_texture(Rect {
            min: Vec2::new(584.0, 16.0),
            max: Vec2::new(600.0, 32.0),
        });

        UIAssets(texture_atlases.add(atlas))
    }
}

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
                .insert_bundle((Name::new("Score"), ScoreDisplay));

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
                .insert_bundle((Name::new("High Score"), HighScoreDisplay));

            builder
                .spawn_bundle(GridBundle {
                    grid: GRID,
                    location: GridLocation { x: 2, y: 0 },
                    layer: Layer::UI,
                    ..default()
                })
                .insert_bundle(SpatialBundle::default())
                .insert_bundle((Name::new("Lives"), LivesDisplay));
        });
}

fn update_score_display(score: ResMut<Score>, mut query: Query<&mut Text, With<ScoreDisplay>>) {
    if !score.is_changed() {
        return;
    }

    for mut text in &mut query {
        text.string = score.to_string();
    }
}

fn update_high_score_display(
    high_score: ResMut<HighScore>,
    mut query: Query<&mut Text, With<HighScoreDisplay>>,
) {
    if !high_score.is_changed() {
        return;
    }

    for mut text in &mut query {
        text.string = high_score.to_string();
    }
}

fn update_lives_display(
    mut commands: Commands,
    lives: Res<Lives>,
    ui_assets: Res<UIAssets>,
    mut query: Query<(Entity, Option<&Children>), With<LivesDisplay>>,
) {
    if !lives.is_changed() {
        return;
    }

    for (entity, children) in &mut query {
        let num_lives = **lives;
        let num_displayed_lives = children.map(|c| c.len()).unwrap_or(0);

        match num_displayed_lives.cmp(&num_lives) {
            Ordering::Greater => {
                for lost_life in &children.unwrap()[num_lives..] {
                    commands.entity(*lost_life).despawn_recursive();
                }
            }
            Ordering::Less => commands.entity(entity).add_children(|bldr| {
                for new_life in num_displayed_lives..num_lives {
                    bldr.spawn_bundle(SpriteSheetBundle {
                        texture_atlas: ui_assets.clone(),
                        sprite: TextureAtlasSprite {
                            index: 0,
                            anchor: Anchor::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .insert_bundle(GridBundle {
                        grid: Grid {
                            size: Vec2::splat(GRID_SIZE * 2.0),
                            offset: Vec2::splat(GRID_SIZE / 2.0),
                        },
                        location: GridLocation {
                            x: new_life as isize,
                            y: 0,
                        },
                        ..default()
                    })
                    .insert(Name::new("Life"));
                }
            }),
            Ordering::Equal => {}
        }
    }
}
