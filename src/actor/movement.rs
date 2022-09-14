use crate::grid::{GridLocation, GridMoving, SetGridLocation, SetGridMoving, Speed};
use crate::layout::Layout;
use crate::WIDTH_TILES;
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use std::time::Duration;

pub const BASE_SPEED: Speed = Speed(88.0);

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate.in_ambiguity_set(Animate))
            .add_system(set_sprite_direction.in_ambiguity_set(Animate).after(SetDir))
            .add_system(change_to_next_dir.label(SetDir).after(SetNextDir))
            .add_system(
                move_dir
                    .label(SetGridMoving)
                    .after(SetDir)
                    .after(SetGridLocation),
            )
            .add_system(wrap_left_right.label(SetGridLocation).before(SetGridMoving));
    }
}

#[derive(AmbiguitySetLabel)]
struct Animate;

#[derive(SystemLabel)]
pub struct SetDir;

#[derive(SystemLabel)]
pub struct SetNextDir;

#[derive(Bundle, Default)]
pub struct MovementBundle {
    pub animation_timer: AnimationTimer,
    pub speed: Speed,
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug, Inspectable)]
pub enum Dir {
    Left,
    Right,
    Down,
    Up,
}

#[derive(Debug, Component, Default, Deref, DerefMut, Inspectable)]
pub struct NextDir(Option<Dir>);

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
        },
        StartLocation(location),
    )
}

fn move_dir(
    mut commands: Commands,
    layout: Res<Layout>,
    query: Query<(Entity, &GridLocation, &Dir), Without<GridMoving>>,
) {
    for (entity, location, dir) in &query {
        let new_loc = location.shift(*dir);
        if !layout.collides(&new_loc) {
            commands.entity(entity).insert(GridMoving {
                destination: new_loc,
                ..default()
            });
        }
    }
}

fn change_to_next_dir(
    layout: Res<Layout>,
    mut query: Query<(&GridLocation, &NextDir, &mut Dir), Without<GridMoving>>,
) {
    for (location, next, mut dir) in &mut query {
        if let Some(next) = next.0 {
            if !layout.collides(&location.shift(next)) {
                *dir = next;
            }
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
        let new_x = (location.x + MARGIN).rem_euclid(WIDTH_TILES as isize + MARGIN * 2) - MARGIN;
        // Check so we don't trigger change detection
        if location.x != new_x {
            location.x = new_x;
        }
    }
}
