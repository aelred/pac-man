use crate::grid::{GridLocation, GridMoving};
use crate::layout::{Layout, Tile};
use crate::player::PlayerMovement;
use crate::WIDTH_TILES;
use bevy::prelude::*;
use bevy::utils::HashSet;
use rand::prelude::SliceRandom;
use std::time::Duration;

const MOVE_DURATION: Duration = Duration::from_millis(90);

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_dir.after(PlayerMovement))
            .add_system(animate)
            .add_system(set_sprite_direction)
            .add_system(move_random)
            .add_system(wrap_left_right);
    }
}

#[derive(Bundle, Default)]
pub struct MovementBundle {
    pub animation_timer: AnimationTimer,
    pub collides: Collides,
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Dir {
    Left,
    Right,
    Down,
    Up,
}

impl Dir {
    fn reverse(&self) -> Dir {
        match self {
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
            Dir::Down => Dir::Up,
            Dir::Up => Dir::Down,
        }
    }
}

#[derive(Component)]
pub struct MoveRandom;

#[derive(Component, Default, Deref, DerefMut)]
pub struct Collides(pub HashSet<Tile>);

impl Collides {
    pub fn at(&self, layout: &Layout, loc: &GridLocation) -> bool {
        layout.get(loc).map_or(false, |tile| self.contains(&tile))
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(200), true))
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct StartLocation(GridLocation);

pub fn moving_left(location: GridLocation) -> impl Bundle {
    (
        location,
        Dir::Left,
        GridMoving {
            destination: location.shift(Dir::Left),
            progress: 0.5,
            duration: MOVE_DURATION,
        },
        StartLocation(location),
    )
}

fn move_random(
    layout: Res<Layout>,
    mut query: Query<(&GridLocation, &Collides, &mut Dir), (With<MoveRandom>, Without<GridMoving>)>,
) {
    let mut rng = rand::thread_rng();

    for (location, collides, mut current_dir) in &mut query {
        let mut dirs: Vec<Dir> = Vec::new();
        for dir in &[Dir::Left, Dir::Right, Dir::Up, Dir::Down] {
            if !collides.at(&layout, &location.shift(*dir)) && *dir != current_dir.reverse() {
                dirs.push(*dir);
            }
        }
        if let Some(new_dir) = dirs.choose(&mut rng) {
            *current_dir = *new_dir;
        }
    }
}

fn move_dir(
    mut commands: Commands,
    layout: Res<Layout>,
    mut query: Query<(Entity, &GridLocation, &Collides, &Dir), Without<GridMoving>>,
) {
    for (entity, location, collides, dir) in &mut query {
        let new_loc = location.shift(*dir);
        if !collides.at(&layout, &new_loc) {
            commands.entity(entity).insert(GridMoving {
                destination: new_loc,
                duration: MOVE_DURATION,
                ..default()
            });
        }
    }
}

fn animate(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlasSprite, &mut AnimationTimer), With<GridMoving>>,
) {
    for (mut sprite, mut timer) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            // Toggle last bit
            sprite.index ^= 1;
        }
    }
}

fn set_sprite_direction(mut query: Query<(&Dir, &mut TextureAtlasSprite), Changed<Dir>>) {
    for (dir, mut sprite) in &mut query {
        let index = match dir {
            Dir::Right => 0,
            Dir::Left => 2,
            Dir::Up => 4,
            Dir::Down => 6,
        };
        // Leave the last bit unchanged so we don't disrupt the animation
        sprite.index = index | (sprite.index & 1);
    }
}

fn wrap_left_right(mut query: Query<&mut GridLocation>) {
    const MARGIN: isize = 2;

    for mut location in &mut query {
        location.x = (location.x + MARGIN).rem_euclid(WIDTH_TILES as isize + MARGIN * 2) - MARGIN;
    }
}
