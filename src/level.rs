use crate::food::{Eater, Food};
use crate::grid::{Grid, GridLocation, Layer};
use crate::movement::{moving_left, Collides, MoveRandom, MovementBundle};
use crate::player::Player;
use bevy::prelude::*;
use bevy::sprite::Rect;
use bevy::utils::HashSet;

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
    offset: Vec2::new(GRID_SIZE / 2.0 - WIDTH / 2.0, -HEIGHT / 2.0),
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Layout::load())
            .add_startup_system(setup_background)
            .add_startup_system(setup_level);
    }
}

pub struct Layout([[Option<Tile>; HEIGHT_TILES]; WIDTH_TILES]);

impl Layout {
    fn load() -> Layout {
        let mut tiles = [[None; HEIGHT_TILES]; WIDTH_TILES];

        let level_bmp = include_bytes!("../assets/level.bmp");

        // Find start of image data
        const OFFSET: usize = 0x0A;
        let offset = u32::from_le_bytes(level_bmp[OFFSET..OFFSET + 4].try_into().unwrap()) as usize;

        let mut img_data = level_bmp[offset..].iter();

        for y in 0..HEIGHT_TILES {
            for x in 0..WIDTH_TILES {
                let tile = match img_data.next().unwrap() {
                    0x01 => Some(Tile::Wall),
                    0x02 => Some(Tile::Dot),
                    0x04 => Some(Tile::Energizer),
                    0x05 => Some(Tile::PacMan),
                    0x06 => Some(Tile::Door),
                    0x07 => Some(Tile::Blinky),
                    0x08 => Some(Tile::Pinky),
                    0x09 => Some(Tile::Inky),
                    0x0A => Some(Tile::Clyde),
                    _ => None,
                };
                tiles[x][y] = tile;
            }
        }

        Layout(tiles)
    }

    pub fn get(&self, loc: &GridLocation) -> Option<Tile> {
        self.0[(loc.x.max(0) as usize).min(WIDTH_TILES - 1)][loc.y as usize]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Tile {
    Wall,
    Door,
    Dot,
    Energizer,
    PacMan,
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let sheet = asset_server.load("sprite_sheet.png");

    let mut background_atlas = TextureAtlas::new_empty(sheet, Vec2::new(680.0, 248.0));

    let start = Vec2::new(228.0, 0.0);
    let background = background_atlas.add_texture(Rect {
        min: start,
        max: start
            + Vec2::new(
                WIDTH,
                HEIGHT - (TOP_MARGIN + BOTTOM_MARGIN) as f32 * GRID_SIZE,
            ),
    });

    let background_handle = texture_atlases.add(background_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(
                0.0,
                -GRID_SIZE * BOTTOM_MARGIN as f32 / 2.0,
                0.0,
            )),
            sprite: TextureAtlasSprite::new(background),
            texture_atlas: background_handle.clone(),
            ..default()
        })
        .insert(Name::new("Background"));
}

fn setup_level(
    mut commands: Commands,
    layout: Res<Layout>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let sheet = asset_server.load("sprite_sheet.png");

    let pacman_atlas = TextureAtlas::from_grid_with_padding(
        sheet.clone(),
        Vec2::splat(16.0),
        2,
        4,
        Vec2::ZERO,
        Vec2::new(456.0, 0.0),
    );

    let blinky_atlas = TextureAtlas::from_grid_with_padding(
        sheet.clone(),
        Vec2::splat(16.0),
        8,
        1,
        Vec2::ZERO,
        Vec2::new(456.0, 64.0),
    );

    let pinky_atlas = TextureAtlas::from_grid_with_padding(
        sheet.clone(),
        Vec2::splat(16.0),
        8,
        1,
        Vec2::ZERO,
        Vec2::new(456.0, 80.0),
    );

    let inky_atlas = TextureAtlas::from_grid_with_padding(
        sheet.clone(),
        Vec2::splat(16.0),
        8,
        1,
        Vec2::ZERO,
        Vec2::new(456.0, 96.0),
    );

    let clyde_atlas = TextureAtlas::from_grid_with_padding(
        sheet.clone(),
        Vec2::splat(16.0),
        8,
        1,
        Vec2::ZERO,
        Vec2::new(456.0, 112.0),
    );

    let mut static_atlas = TextureAtlas::new_empty(sheet, Vec2::new(680.0, 248.0));

    let dot = static_atlas.add_texture(Rect {
        min: Vec2::new(8.0, 8.0),
        max: Vec2::new(16.0, 16.0),
    });
    let energizer = static_atlas.add_texture(Rect {
        min: Vec2::new(8.0, 24.0),
        max: Vec2::new(16.0, 32.0),
    });

    let misc_handle = texture_atlases.add(static_atlas);
    let pacman_handle = texture_atlases.add(pacman_atlas);
    let blinky_handle = texture_atlases.add(blinky_atlas);
    let pinky_handle = texture_atlases.add(pinky_atlas);
    let inky_handle = texture_atlases.add(inky_atlas);
    let clyde_handle = texture_atlases.add(clyde_atlas);

    commands
        .spawn_bundle(SpatialBundle::default())
        .insert(Name::new("Level"))
        .with_children(|bldr| {
            for x in 0..WIDTH_TILES {
                for y in 0..HEIGHT_TILES {
                    let loc = GridLocation {
                        x: x as isize,
                        y: y as isize,
                    };

                    match layout.0[x][y] {
                        Some(Tile::Dot) => {
                            spawn_food(bldr, loc, "Dot", dot, &misc_handle, 10);
                        }
                        Some(Tile::Energizer) => {
                            spawn_food(bldr, loc, "Energizer", energizer, &misc_handle, 50);
                        }
                        Some(Tile::PacMan) => spawn_pacman(bldr, loc, &pacman_handle),
                        Some(Tile::Blinky) => spawn_ghost(bldr, loc, "Blinky", &blinky_handle),
                        Some(Tile::Pinky) => spawn_ghost(bldr, loc, "Pinky", &pinky_handle),
                        Some(Tile::Inky) => spawn_ghost(bldr, loc, "Inky", &inky_handle),
                        Some(Tile::Clyde) => spawn_ghost(bldr, loc, "Clyde", &clyde_handle),
                        _ => {}
                    }
                }
            }
        });
}

#[derive(Bundle, Default)]
struct GridEntity {
    name: Name,
    sprite: TextureAtlasSprite,
    texture_atlas: Handle<TextureAtlas>,
    grid: Grid,
    location: GridLocation,
    layer: Layer,
    #[bundle]
    _spatial: SpatialBundle,
}

fn spawn_pacman(commands: &mut ChildBuilder, location: GridLocation, atlas: &Handle<TextureAtlas>) {
    commands
        .spawn_bundle(GridEntity {
            name: Name::new("Pac-Man"),
            texture_atlas: atlas.clone(),
            grid: GRID,
            location,
            ..default()
        })
        .insert_bundle(MovementBundle {
            collides: Collides(HashSet::from([Tile::Wall, Tile::Door])),
            ..default()
        })
        .insert_bundle(moving_left(location))
        .insert_bundle((Player::default(), Eater));
}

fn spawn_food(
    commands: &mut ChildBuilder,
    location: GridLocation,
    name: &'static str,
    sprite_index: usize,
    atlas: &Handle<TextureAtlas>,
    points: u32,
) -> Entity {
    commands
        .spawn_bundle(GridEntity {
            name: Name::new(name),
            sprite: TextureAtlasSprite::new(sprite_index),
            texture_atlas: atlas.clone(),
            grid: GRID,
            location,
            layer: Layer(4.0),
            ..default()
        })
        .insert(Food { points })
        .id()
}

fn spawn_ghost(
    commands: &mut ChildBuilder,
    location: GridLocation,
    name: &'static str,
    atlas: &Handle<TextureAtlas>,
) {
    commands
        .spawn_bundle(GridEntity {
            name: Name::new(name),
            texture_atlas: atlas.clone(),
            grid: GRID,
            location,
            ..default()
        })
        .insert_bundle(MovementBundle {
            collides: Collides(HashSet::from([Tile::Wall])),
            ..default()
        })
        .insert_bundle(moving_left(location))
        .insert(MoveRandom);
}
