use crate::food::{Eater, Energizer, Food};
use crate::ghost::{Blinky, Clyde, GhostSpawner, Inky, Pinky};
use crate::grid::{Grid, GridBundle, GridLocation, Layer};
use crate::layout::{Layout, Tile};
use crate::movement::{moving_left, MovementBundle, NextDir, StartLocation};
use crate::player::{Player, PlayerDied};
use bevy::prelude::*;
use bevy::sprite::Rect;

pub const WIDTH_TILES: usize = 28;
pub const HEIGHT_TILES: usize = 36;
pub const TOP_MARGIN: usize = 3;
pub const BOTTOM_MARGIN: usize = 2;
pub const WIDTH: f32 = WIDTH_TILES as f32 * GRID_SIZE;
pub const HEIGHT: f32 = HEIGHT_TILES as f32 * GRID_SIZE;
pub const SCALE: f32 = 3.0;
pub const GRID_SIZE: f32 = 8.0;

pub const GRID: Grid = Grid {
    size: Vec2::splat(GRID_SIZE),
    offset: Vec2::ZERO,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelAssets>()
            .add_startup_system(setup_background)
            .add_startup_system(create_level)
            .add_system(reset_level_when_player_dies);
    }
}

#[derive(Component)]
struct Level;

struct LevelAssets {
    pac_man: Handle<TextureAtlas>,
    objects: Handle<TextureAtlas>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server: &AssetServer = world.resource();
        let sheet = asset_server.load("sprite_sheet.png");

        let mut texture_atlases: Mut<Assets<TextureAtlas>> = world.resource_mut();

        let pac_man_atlas = TextureAtlas::from_grid_with_padding(
            sheet.clone(),
            Vec2::splat(16.0),
            2,
            4,
            Vec2::ZERO,
            Vec2::new(456.0, 0.0),
        );

        let mut object_atlas = TextureAtlas::new_empty(sheet, Vec2::new(680.0, 248.0));

        object_atlas.add_texture(Rect {
            min: Vec2::new(8.0, 8.0),
            max: Vec2::new(16.0, 16.0),
        });
        object_atlas.add_texture(Rect {
            min: Vec2::new(8.0, 24.0),
            max: Vec2::new(16.0, 32.0),
        });

        Self {
            pac_man: texture_atlases.add(pac_man_atlas),
            objects: texture_atlases.add(object_atlas),
        }
    }
}

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let sheet = asset_server.load("sprite_sheet.png");

    let mut background_atlas = TextureAtlas::new_empty(sheet, Vec2::new(680.0, 248.0));

    let start = Vec2::new(228.0, 0.0);

    const BACKGROUND_HEIGHT: f32 = HEIGHT - (TOP_MARGIN + BOTTOM_MARGIN) as f32 * GRID_SIZE;

    background_atlas.add_texture(Rect {
        min: start,
        max: start + Vec2::new(WIDTH, BACKGROUND_HEIGHT),
    });

    let background_handle = texture_atlases.add(background_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: background_handle,
            ..default()
        })
        .insert_bundle(GridBundle {
            grid: Grid {
                size: Vec2::splat(GRID_SIZE),
                offset: Vec2::new(
                    (WIDTH - GRID_SIZE) / 2.0,
                    (BACKGROUND_HEIGHT - GRID_SIZE) / 2.0,
                ),
            },
            location: GridLocation {
                x: 0,
                y: BOTTOM_MARGIN as isize,
            },
            layer: Layer::BACKGROUND,
            ..default()
        })
        .insert(Name::new("Background"));
}

fn create_level(
    mut commands: Commands,
    layout: Res<Layout>,
    level_assets: Res<LevelAssets>,
    ghosts: Res<GhostSpawner>,
) {
    commands
        .spawn_bundle(SpatialBundle::default())
        .insert_bundle((Name::new("Level"), Level))
        .with_children(|bldr| spawn_level_entities(bldr, &layout, &level_assets, &ghosts));
}

fn reset_level_when_player_dies(
    mut commands: Commands,
    mut died_events: EventReader<PlayerDied>,
    query: Query<(Entity, &StartLocation)>,
    mut player: Query<&mut NextDir, With<Player>>,
) {
    for _ in died_events.iter() {
        for (entity, start_location) in &query {
            commands
                .entity(entity)
                .insert_bundle(moving_left(**start_location));
        }

        for mut player in &mut player {
            **player = None;
        }
    }
}

fn spawn_level_entities(
    bldr: &mut ChildBuilder,
    layout: &Layout,
    level_assets: &LevelAssets,
    ghosts: &GhostSpawner,
) {
    for x in 0..WIDTH_TILES {
        for y in 0..HEIGHT_TILES {
            let loc = GridLocation {
                x: x as isize,
                y: y as isize,
            };

            match layout.get(&loc) {
                Some(Tile::Dot) => {
                    spawn_food(bldr, loc, "Dot", 0, level_assets, 10, ());
                }
                Some(Tile::Energizer) => {
                    spawn_food(bldr, loc, "Energizer", 1, level_assets, 50, (Energizer,));
                }
                Some(Tile::PacMan) => spawn_pac_man(bldr, loc, level_assets),
                Some(Tile::Blinky) => ghosts.spawn(bldr, Blinky, loc),
                Some(Tile::Pinky) => ghosts.spawn(bldr, Pinky, loc),
                Some(Tile::Inky) => ghosts.spawn(bldr, Inky, loc),
                Some(Tile::Clyde) => ghosts.spawn(bldr, Clyde, loc),
                _ => {}
            }
        }
    }
}

#[derive(Bundle, Default)]
pub struct GridEntity {
    pub name: Name,
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
    pub grid: Grid,
    pub location: GridLocation,
    pub layer: Layer,
    #[bundle]
    pub _spatial: SpatialBundle,
}

fn spawn_pac_man(commands: &mut ChildBuilder, location: GridLocation, level_assets: &LevelAssets) {
    commands
        .spawn_bundle(GridEntity {
            name: Name::new("Pac-Man"),
            texture_atlas: level_assets.pac_man.clone(),
            grid: GRID,
            location,
            ..default()
        })
        .insert_bundle(MovementBundle::default())
        .insert_bundle(moving_left(location))
        .insert_bundle((NextDir::default(), Player, Eater));
}

fn spawn_food(
    commands: &mut ChildBuilder,
    location: GridLocation,
    name: &'static str,
    sprite_index: usize,
    level_assets: &LevelAssets,
    points: u32,
    bundle: impl Bundle,
) {
    commands
        .spawn_bundle(GridEntity {
            name: Name::new(name),
            sprite: TextureAtlasSprite::new(sprite_index),
            texture_atlas: level_assets.objects.clone(),
            grid: GRID,
            location,
            layer: Layer(4.0),
            ..default()
        })
        .insert(Food { points })
        .insert_bundle(bundle);
}
