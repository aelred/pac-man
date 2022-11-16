use bevy::{math::Rect, prelude::*};

use crate::{
    actor::movement::{moving_left, MovementBundle, NextDir, BASE_SPEED},
    grid::{GridBundle, GridLocation},
    level::{GridEntity, GRID},
};

use super::{Ghost, GhostSpawner, PersonalityT, ScatterTarget, Target};

impl GhostSpawner {
    pub fn spawn<P: PersonalityT>(
        &self,
        bldr: &mut ChildBuilder,
        personality: P,
        location: GridLocation,
    ) {
        bldr.spawn(GridEntity {
            name: Name::new(P::NAME),
            texture_atlas: self.get_atlas(P::VALUE),
            grid: GridBundle::new(GRID, location, default()),
            ..default()
        })
        .insert(moving_left(location))
        .insert(GhostBundle {
            scatter_target: ScatterTarget(P::SCATTER),
            ghost: Ghost {
                personality: P::VALUE,
            },
            personality,
            movement: MovementBundle {
                speed: BASE_SPEED * 0.75,
                ..default()
            },
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

        let blinky_atlas = TextureAtlas::from_grid(
            sheet.clone(),
            tile_size,
            8,
            1,
            None,
            Some(Vec2::new(456.0, 64.0)),
        );

        let pinky_atlas = TextureAtlas::from_grid(
            sheet.clone(),
            Vec2::splat(16.0),
            8,
            1,
            None,
            Some(Vec2::new(456.0, 80.0)),
        );

        let inky_atlas = TextureAtlas::from_grid(
            sheet.clone(),
            Vec2::splat(16.0),
            8,
            1,
            None,
            Some(Vec2::new(456.0, 96.0)),
        );

        let clyde_atlas = TextureAtlas::from_grid(
            sheet.clone(),
            Vec2::splat(16.0),
            8,
            1,
            None,
            Some(Vec2::new(456.0, 112.0)),
        );

        let mut frightened_atlas = TextureAtlas::new_empty(sheet.clone(), Vec2::new(680.0, 248.0));
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

        let mut respawning_atlas = TextureAtlas::new_empty(sheet, Vec2::new(680.0, 248.0));
        // Add each sprite twice (because the respawning eyes don't animate)
        for ix in 0..4 {
            for _ in 0..2 {
                let min = Vec2::new(584.0 + ix as f32 * 16.0, 80.0);
                respawning_atlas.add_texture(Rect {
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
            respawning: texture_atlases.add(respawning_atlas),
        }
    }
}

#[derive(Bundle, Default)]
struct GhostBundle<P: PersonalityT> {
    pub scatter_target: ScatterTarget,
    pub ghost: Ghost,
    pub personality: P,
    pub next_dir: NextDir,
    #[bundle]
    pub movement: MovementBundle,
    pub _target: Target,
}
