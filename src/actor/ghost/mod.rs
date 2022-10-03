mod assets;
mod blinky;
mod clyde;
mod inky;
mod pinky;

use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    actor::mode::{FrightenedMode, Mode, SetMode, TickMode},
    actor::movement::{Dir, NextDir, StartLocation, BASE_SPEED},
    food::{Eat, Food, WriteEatEvent},
    grid::{GridLocation, SetGridLocation},
    layout::Layout,
};

pub use blinky::Blinky;
pub use clyde::Clyde;
pub use inky::Inky;
pub use pinky::Pinky;

use super::{
    movement::{MovementAmbiguity, SetDir, SetNextDir},
    player::Player,
};

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GhostSpawner>()
            .add_system_set(
                SystemSet::new()
                    .label(SetNextDir)
                    .in_ambiguity_set(MovementAmbiguity)
                    .after(TickMode)
                    .before(SetGridLocation)
                    .with_system(choose_next_dir)
                    .with_system(frightened),
            )
            .add_system_set(
                SystemSet::new()
                    .label(SetTarget)
                    .in_ambiguity_set(MovementAmbiguity)
                    .after(SetMode)
                    .after(SetGridLocation)
                    .with_system(blinky::chase)
                    .with_system(pinky::chase.after(SetDir))
                    .with_system(inky::chase.after(SetDir))
                    .with_system(clyde::chase)
                    .with_system(scatter),
            )
            .add_system(become_frightened.after(SetMode))
            .add_system(stop_frightened.after(SetMode))
            .add_system(
                start_respawning_eaten_ghost
                    .after(WriteEatEvent)
                    .after(SetTarget),
            )
            .add_system(finish_respawning_eaten_ghost.after(SetTarget));
    }
}

#[derive(SystemLabel)]
pub struct SetTarget;

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

pub type ActiveGhost = (With<Ghost>, Without<Respawning>, Without<Frightened>);

const DIRECTIONS: [Dir; 4] = [Dir::Up, Dir::Left, Dir::Down, Dir::Right];

// Ghosts decide on their next direction one grid location BEFORE
fn choose_next_dir(
    layout: Res<Layout>,
    mut query: Query<
        (&Dir, &mut NextDir, &GridLocation, &Target),
        (Changed<GridLocation>, Without<Player>, Without<Frightened>),
    >,
) {
    for (dir, mut next_dir, loc, target) in &mut query {
        let next_loc = loc.shift(*dir);

        if layout.collides(&next_loc) {
            continue;
        }

        **next_dir = closest_dir_to_target(&layout, next_loc, *target, Some(*loc));
    }
}

fn scatter(
    mode: Res<Mode>,
    mut query: Query<(Option<&Name>, &ScatterTarget, &mut Target), ActiveGhost>,
) {
    if *mode != Mode::Scatter {
        return;
    }

    for (name, scatter_target, mut target) in &mut query {
        if **target != **scatter_target {
            debug!(
                "{} scattering: changing target from {:?} to {:?}",
                name.unwrap_or(&Name::new("<Unknown>")),
                **target,
                **scatter_target
            );
            **target = **scatter_target;
        }
    }
}

fn become_frightened(
    mut commands: Commands,
    mode: Res<FrightenedMode>,
    assets: Res<GhostSpawner>,
    mut query: Query<Entity, ActiveGhost>,
) {
    if !mode.is_changed() || *mode == FrightenedMode::Disabled {
        return;
    }

    for entity in &mut query {
        commands
            .entity(entity)
            .insert_bundle(FrightenedBundle {
                food: Food { points: 200 },
                ..default()
            })
            .insert(BASE_SPEED * 0.5)
            .insert(assets.frightened.clone())
            .remove::<Target>();
    }
}

fn stop_frightened(
    mut commands: Commands,
    mode: Res<FrightenedMode>,
    assets: Res<GhostSpawner>,
    mut query: Query<(Entity, &Ghost), With<Frightened>>,
) {
    if !mode.is_changed() || *mode == FrightenedMode::Enabled {
        return;
    }

    for (entity, ghost) in &mut query {
        commands
            .entity(entity)
            .remove_bundle::<FrightenedBundle>()
            .insert(BASE_SPEED * 0.75)
            .insert(assets.get_atlas(ghost))
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
    ghosts: Query<(&GridLocation, &StartLocation), With<Ghost>>,
) {
    for Eat(eaten) in eat_events.iter() {
        if let Ok((location, start)) = ghosts.get(*eaten) {
            let target = Target(**start);

            commands
                .entity(*eaten)
                .insert(Respawning)
                .insert(target)
                // This speed is just a guess
                .insert(BASE_SPEED * 2.0)
                .insert(assets.respawning.clone())
                .remove_bundle::<FrightenedBundle>();

            if let Some(dir) = closest_dir_to_target(&layout, *location, target, None) {
                commands.entity(*eaten).insert(dir);
            }
        }
    }
}

fn finish_respawning_eaten_ghost(
    mut commands: Commands,
    assets: Res<GhostSpawner>,
    mut respawning: Query<
        (Entity, &Ghost, &StartLocation, &GridLocation),
        (With<Respawning>, Changed<GridLocation>),
    >,
) {
    for (entity, ghost, start, location) in &mut respawning {
        if **start == *location {
            commands
                .entity(entity)
                .insert(BASE_SPEED * 0.75)
                .insert(assets.get_atlas(ghost))
                .remove::<Respawning>();
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
