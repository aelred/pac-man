use bevy::{prelude::*, sprite::Rect};

use crate::{
    grid::GridLocation,
    level::{GridEntity, GRID},
    movement::{moving_left, MovementBundle, NextDir},
};

use super::{Ghost, GhostSpawner, Personality, ScatterTarget, Target};

impl GhostSpawner {
    pub fn spawn<P: Personality>(
        &self,
        bldr: &mut ChildBuilder,
        personality: P,
        location: GridLocation,
    ) {
        bldr.spawn_bundle(GridEntity {
            name: Name::new(P::NAME),
            texture_atlas: P::get_atlas(self).clone(),
            grid: GRID,
            location,
            ..default()
        })
        .insert_bundle(moving_left(location))
        .insert_bundle(GhostBundle {
            scatter_target: ScatterTarget(P::SCATTER),
            personality,
            ..default()
        });
    }
}

impl FromWorld for GhostSpawner {
    fn from_world(world: &mut World) -> Self {
        let asset_server: &AssetServer = world.resource();
        let sheet = asset_server.load("sprite_sheet.png");

        let mut texture_atlases: Mut<Assets<TextureAtlas>> = world.resource_mut();

        let tile_size = Vec2::splat(16.0);

        let blinky_atlas = TextureAtlas::from_grid_with_padding(
            sheet.clone(),
            tile_size,
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

        let mut frightened_atlas = TextureAtlas::new_empty(sheet, Vec2::new(680.0, 248.0));
        // Add same two sprites four times (the sprites are the same for each direction
        for _ in 0..4 {
            for ix in 0..2 {
                let min = Vec2::new(584.0 + ix as f32 * 16.0, 64.0);
                frightened_atlas.add_texture(Rect {
                    min,
                    max: min + tile_size,
                });
            }
        }

        Self {
            blinky: texture_atlases.add(blinky_atlas),
            pinky: texture_atlases.add(pinky_atlas),
            inky: texture_atlases.add(inky_atlas),
            clyde: texture_atlases.add(clyde_atlas),
            frightened: texture_atlases.add(frightened_atlas),
        }
    }
}

#[derive(Bundle, Default)]
struct GhostBundle<P: Personality> {
    pub scatter_target: ScatterTarget,
    pub ghost: Ghost,
    pub personality: P,
    pub next_dir: NextDir,
    #[bundle]
    pub movement: MovementBundle,
    pub _target: Target,
}