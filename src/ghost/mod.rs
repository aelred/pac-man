mod assets;
mod blinky;
mod clyde;
mod inky;
mod pinky;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    food::{Eat, Food},
    grid::{GridLocation, SetGridLocation, Speed},
    layout::Layout,
    mode::{Mode, SetMode, TickMode},
    movement::{Dir, NextDir, SetDir, StartLocation, BASE_SPEED},
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
            .add_system(become_frightened.after(SetMode))
            .add_system(stop_frightened.after(SetMode))
            .add_system(start_respawning_eaten_ghost.after(SetTarget))
            .add_system(finish_respawning_eaten_ghost.after(SetTarget));
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
    respawning: Handle<TextureAtlas>,
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

#[derive(Component, Default, Deref, DerefMut, Copy, Clone)]
pub struct Target(pub GridLocation);

#[derive(Component, Default, Deref, DerefMut)]
struct ScatterTarget(pub GridLocation);

#[derive(Component, Default)]
pub struct Frightened;

#[derive(Component)]
pub struct Respawning;

#[derive(Bundle, Default)]
struct FrightenedBundle {
    frightened: Frightened,
    food: Food,
}

const DIRECTIONS: [Dir; 4] = [Dir::Up, Dir::Left, Dir::Down, Dir::Right];

// Ghosts decide on their next direction one grid location BEFORE
fn choose_next_dir(
    layout: Res<Layout>,
    mut query: Query<(&Dir, &mut NextDir, &GridLocation, &Target), Changed<GridLocation>>,
) {
    for (dir, mut next_dir, loc, target) in &mut query {
        let next_loc = loc.shift(*dir);

        if layout.collides(&next_loc) {
            continue;
        }

        **next_dir = closest_dir_to_target(&layout, next_loc, *target, Some(*loc));
    }
}

fn scatter(mode: Res<Mode>, mut query: Query<(&ScatterTarget, &mut Target), Without<Respawning>>) {
    if *mode != Mode::Scatter {
        return;
    }

    for (scatter_target, mut target) in &mut query {
        if **target != **scatter_target {
            **target = **scatter_target;
        }
    }
}

fn become_frightened(
    mut commands: Commands,
    mode: Res<Mode>,
    assets: Res<GhostSpawner>,
    mut query: Query<
        (Entity, &mut Handle<TextureAtlas>, &mut Speed),
        (With<Ghost>, Without<Respawning>),
    >,
) {
    if !mode.is_changed() || *mode != Mode::Frightened {
        return;
    }

    for (entity, mut texture, mut speed) in &mut query {
        *speed = BASE_SPEED * 0.5;
        *texture = assets.frightened.clone();

        commands
            .entity(entity)
            .insert_bundle(FrightenedBundle {
                food: Food { points: 200 },
                ..default()
            })
            .remove::<Target>();
    }
}

fn stop_frightened(
    mut commands: Commands,
    mode: Res<Mode>,
    assets: Res<GhostSpawner>,
    mut query: Query<
        (Entity, &Ghost, &mut Handle<TextureAtlas>, &mut Speed),
        (With<Ghost>, Without<Respawning>),
    >,
) {
    if !mode.is_changed() || *mode == Mode::Frightened {
        return;
    }

    for (entity, ghost, mut texture, mut speed) in &mut query {
        *speed = BASE_SPEED * 0.75;
        *texture = assets.get_atlas(ghost);

        commands
            .entity(entity)
            .remove_bundle::<FrightenedBundle>()
            .insert(Target::default());
    }
}

fn frightened(
    layout: Res<Layout>,
    mut query: Query<
        (&Dir, &mut NextDir, &GridLocation),
        (With<Frightened>, Changed<GridLocation>, Without<Player>),
    >,
) {
    for (dir, mut next_dir, loc) in &mut query {
        let next_loc = loc.shift(*dir);

        if layout.collides(&next_loc) {
            continue;
        }

        // The PRNG generates an pseudo-random memory address to read the last few bits from.
        // These bits are translated into the direction a frightened ghost must first try.
        // If a wall blocks the chosen direction, the ghost then attempts the remaining directions
        // in this order: up, left, down, and right, until a passable direction is found.
        // https://www.gamedeveloper.com/design/the-pac-man-dossier#:~:text=The%20PRNG%20generates,direction%20is%20found.
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

fn start_respawning_eaten_ghost(
    mut commands: Commands,
    layout: Res<Layout>,
    assets: Res<GhostSpawner>,
    mut eat_events: EventReader<Eat>,
    mut ghosts: Query<
        (
            &mut Handle<TextureAtlas>,
            &mut Speed,
            &mut NextDir,
            &GridLocation,
            &StartLocation,
        ),
        With<Ghost>,
    >,
) {
    for Eat(eaten) in eat_events.iter() {
        if let Ok((mut texture, mut speed, mut next_dir, location, start)) = ghosts.get_mut(*eaten)
        {
            let target = Target(**start);

            commands
                .entity(*eaten)
                .insert(Respawning)
                .insert(target)
                .remove_bundle::<FrightenedBundle>();

            *texture = assets.respawning.clone();
            // This speed is just a guess
            *speed = BASE_SPEED * 2.0;
            **next_dir = closest_dir_to_target(&layout, *location, target, None);
        }
    }
}

fn finish_respawning_eaten_ghost(
    mut commands: Commands,
    assets: Res<GhostSpawner>,
    mut respawning: Query<
        (
            Entity,
            &Ghost,
            &StartLocation,
            &GridLocation,
            &mut Speed,
            &mut Handle<TextureAtlas>,
        ),
        (With<Respawning>, Changed<GridLocation>),
    >,
) {
    for (entity, ghost, start, location, mut speed, mut texture) in &mut respawning {
        if **start == *location {
            *speed = BASE_SPEED * 0.75;
            *texture = assets.get_atlas(ghost);

            commands.entity(entity).remove::<Respawning>();
        }
    }
}

fn closest_dir_to_target(
    layout: &Layout,
    source: GridLocation,
    target: Target,
    original_loc: Option<GridLocation>,
) -> Option<Dir> {
    let mut best_distance = f32::MAX;
    let mut best_dir = None;

    for dir in DIRECTIONS {
        let loc = source.shift(dir);
        let collision = layout.collides(&loc);

        let distance = loc
            .to_unscaled_vec2()
            .distance_squared(target.to_unscaled_vec2());

        if !collision && original_loc != Some(loc) && distance < best_distance {
            best_dir = Some(dir);
            best_distance = distance;
        }
    }

    best_dir
}
