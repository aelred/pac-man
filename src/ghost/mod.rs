mod assets;
mod blinky;
mod clyde;
mod inky;
mod pinky;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    grid::{GridLocation, MoveOnGrid},
    layout::Layout,
    mode::Mode,
    movement::{Dir, NextDir, SetDir},
};

pub use blinky::Blinky;
pub use clyde::Clyde;
pub use inky::Inky;
pub use pinky::Pinky;

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GhostSpawner>()
            .add_system(choose_next_dir.after(SetDir).before(MoveOnGrid))
            .add_system(blinky::chase)
            .add_system(pinky::chase)
            .add_system(inky::chase)
            .add_system(clyde::chase)
            .add_system(scatter)
            .add_system(frightened)
            .add_system(frightened_sprites::<Blinky>)
            .add_system(frightened_sprites::<Pinky>)
            .add_system(frightened_sprites::<Inky>)
            .add_system(frightened_sprites::<Clyde>);
    }
}

pub struct GhostSpawner {
    blinky: Handle<TextureAtlas>,
    pinky: Handle<TextureAtlas>,
    inky: Handle<TextureAtlas>,
    clyde: Handle<TextureAtlas>,
    frightened: Handle<TextureAtlas>,
}

pub trait Personality: Component + Default {
    const NAME: &'static str;
    const COLOR: Color;
    const SCATTER: GridLocation;

    fn get_atlas(assets: &GhostSpawner) -> &Handle<TextureAtlas>;
}

#[derive(Component, Default)]
pub struct Ghost;

#[derive(Component, Default, Deref, DerefMut)]
pub struct Target(pub GridLocation);

#[derive(Component, Default, Deref, DerefMut)]
struct ScatterTarget(pub GridLocation);

const DIRECTIONS: [Dir; 4] = [Dir::Up, Dir::Left, Dir::Down, Dir::Right];

// Ghosts decide on their next direction one grid location BEFORE
fn choose_next_dir(
    mode: Res<Mode>,
    layout: Res<Layout>,
    mut query: Query<(&Dir, &mut NextDir, &GridLocation, &Target), Changed<GridLocation>>,
) {
    if *mode == Mode::Frightened {
        return;
    }

    for (dir, mut next_dir, loc, target) in &mut query {
        let next_loc = loc.shift(*dir);

        if layout.collides(&next_loc) {
            continue;
        }

        let mut distance_to_target = f32::MAX;

        for candidate_dir in DIRECTIONS {
            let candidate_loc = next_loc.shift(candidate_dir);
            let collision = layout.collides(&candidate_loc);

            let distance = candidate_loc
                .to_unscaled_vec2()
                .distance_squared(target.to_unscaled_vec2());

            if !collision && candidate_loc != *loc && distance < distance_to_target {
                **next_dir = Some(candidate_dir);
                distance_to_target = distance;
            }
        }
    }
}

fn scatter(mode: Res<Mode>, mut query: Query<(&ScatterTarget, &mut Target)>) {
    if *mode != Mode::Scatter {
        return;
    }

    for (scatter_target, mut target) in &mut query {
        if **target != **scatter_target {
            **target = **scatter_target;
        }
    }
}

fn frightened_sprites<P: Personality>(
    mode: Res<Mode>,
    assets: Res<GhostSpawner>,
    mut query: Query<&mut Handle<TextureAtlas>, With<P>>,
) {
    if !mode.is_changed() {
        return;
    }

    for mut ghost_atlas in &mut query {
        *ghost_atlas = if *mode == Mode::Frightened {
            assets.frightened.clone()
        } else {
            P::get_atlas(&assets).clone()
        };
    }
}

fn frightened(
    mode: Res<Mode>,
    layout: Res<Layout>,
    mut query: Query<(&Dir, &mut NextDir, &GridLocation), (With<Ghost>, Changed<GridLocation>)>,
) {
    if *mode != Mode::Frightened {
        return;
    }

    for (dir, mut next_dir, loc) in &mut query {
        let next_loc = loc.shift(*dir);

        if layout.collides(&next_loc) {
            continue;
        }

        let random_dir = *DIRECTIONS.choose(&mut rand::thread_rng()).unwrap();

        for candidate_dir in std::iter::once(random_dir).chain(DIRECTIONS) {
            let candidate_loc = next_loc.shift(candidate_dir);
            let collision = layout.collides(&candidate_loc);

            if !collision && candidate_loc != *loc {
                **next_dir = Some(candidate_dir);
                break;
            }
        }
    }
}
