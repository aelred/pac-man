mod assets;
mod blinky;
mod clyde;
mod inky;
mod pinky;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    grid::{GridLocation, SetGridLocation},
    layout::Layout,
    mode::{Mode, SetMode, TickMode},
    movement::{Dir, NextDir, SetDir},
    player::Player,
};

pub use blinky::Blinky;
pub use clyde::Clyde;
pub use inky::Inky;
pub use pinky::Pinky;

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GhostSpawner>()
            // Don't label these SetNextDir, because they never takes effect this frame
            .add_system_set(
                SystemSet::new()
                    .after(TickMode)
                    .after(SetDir)
                    .before(SetGridLocation)
                    .with_system(choose_next_dir)
                    .with_system(frightened),
            )
            .add_system_set(
                // These systems are actually mutually exclusive - they operate on diff ghosts, or in diff modes
                SystemSet::new()
                    .label(SetTarget)
                    .in_ambiguity_set(GhostModes)
                    .after(SetMode)
                    .with_system(blinky::chase)
                    .with_system(pinky::chase)
                    .with_system(inky::chase)
                    .with_system(clyde::chase)
                    .with_system(scatter.after(SetMode)),
            )
            .add_system(frightened_sprites.after(SetMode));
    }
}

#[derive(SystemLabel)]
pub struct SetTarget;

#[derive(AmbiguitySetLabel)]
struct GhostModes;

pub struct GhostSpawner {
    blinky: Handle<TextureAtlas>,
    pinky: Handle<TextureAtlas>,
    inky: Handle<TextureAtlas>,
    clyde: Handle<TextureAtlas>,
    frightened: Handle<TextureAtlas>,
}

impl GhostSpawner {
    fn get_atlas(&self, ghost: &Ghost) -> Handle<TextureAtlas> {
        match ghost {
            Ghost::Blinky => &self.blinky,
            Ghost::Pinky => &self.pinky,
            Ghost::Inky => &self.inky,
            Ghost::Clyde => &self.clyde,
        }
        .clone()
    }
}

pub trait Personality: Component + Default {
    const NAME: &'static str;
    const COLOR: Color;
    const SCATTER: GridLocation;
    const GHOST: Ghost;
}

#[derive(Component, Default)]
pub enum Ghost {
    #[default]
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

impl Ghost {
    pub fn color(&self) -> Color {
        match self {
            Ghost::Blinky => Blinky::COLOR,
            Ghost::Pinky => Pinky::COLOR,
            Ghost::Inky => Inky::COLOR,
            Ghost::Clyde => Clyde::COLOR,
        }
    }
}

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

fn frightened_sprites(
    mode: Res<Mode>,
    assets: Res<GhostSpawner>,
    mut query: Query<(&Ghost, &mut Handle<TextureAtlas>)>,
) {
    if !mode.is_changed() {
        return;
    }

    for (ghost, mut ghost_atlas) in &mut query {
        *ghost_atlas = if *mode == Mode::Frightened {
            assets.frightened.clone()
        } else {
            assets.get_atlas(ghost)
        };
    }
}

fn frightened(
    mode: Res<Mode>,
    layout: Res<Layout>,
    mut query: Query<
        (&Dir, &mut NextDir, &GridLocation),
        (With<Ghost>, Changed<GridLocation>, Without<Player>),
    >,
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
