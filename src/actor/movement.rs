use crate::grid::{
    Grid, GridLocation, Layer, MovingTo, SetGridLocation, SetGridMoving, SetTransform, Speed,
};
use crate::layout::Layout;
use crate::level::{GRID, GRID_SIZE, WIDTH};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use std::time::Duration;

pub const BASE_SPEED: Speed = Speed(88.0);

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate.in_ambiguity_set(Animate))
            .add_system(set_sprite_direction.in_ambiguity_set(Animate).after(SetDir))
            .add_system(
                change_to_next_dir
                    .label(SetDir)
                    .after(SetGridLocation)
                    .after(SetNextDir),
            )
            .add_system(move_dir.label(SetGridMoving).after(SetDir))
            .add_system(move_to.label(SetTransform).after(SetSpeed))
            .add_system(
                wrap_left_right
                    .label(SetTransform)
                    .before(SetGridLocation)
                    .after(move_to),
            );
    }
}

#[derive(AmbiguitySetLabel)]
struct Animate;

#[derive(SystemLabel)]
pub struct SetDir;

#[derive(SystemLabel)]
pub struct SetNextDir;

#[derive(SystemLabel)]
pub struct SetSpeed;

/// Lots of systems set direction, target, etc. but they can run together because they
/// operate in different circumstances (e.g. player, ghost state, ghost type)
#[derive(AmbiguitySetLabel)]
pub struct MovementAmbiguity;

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
        Transform::from_translation(GRID.to_vec2(location).extend(Layer::FOREGROUND.0)),
        Dir::Left,
        NextDir(None),
        MovingTo(
            GRID.to_vec2(location.shift(Dir::Left))
                .extend(Layer::FOREGROUND.0),
        ),
        StartLocation(location),
    )
}

fn move_dir(
    mut commands: Commands,
    layout: Res<Layout>,
    query: Query<(Entity, &GridLocation, &Grid, &Layer, &Dir), Without<MovingTo>>,
) {
    for (entity, location, grid, layer, dir) in &query {
        let new_loc = location.shift(*dir);
        if !layout.collides(&new_loc) {
            commands
                .entity(entity)
                .insert(MovingTo(grid.to_vec2(new_loc).extend(layer.0)));
        }
    }
}

fn change_to_next_dir(
    layout: Res<Layout>,
    mut query: Query<(&GridLocation, &NextDir, &mut Dir), Without<MovingTo>>,
) {
    for (location, next, mut dir) in &mut query {
        if let Some(next) = next.0 {
            if !layout.collides(&location.shift(next)) && *dir != next {
                *dir = next;
            }
        }
    }
}

fn animate(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlasSprite, &mut AnimationTimer), With<MovingTo>>,
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

fn move_to(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Speed, &mut Transform, &MovingTo)>,
) {
    for (entity, speed, mut transform, moving) in query.iter_mut() {
        let destination = moving.0;
        let direction = destination - transform.translation;
        let distance = direction.length();

        let movement = **speed * time.delta_seconds();

        if distance < movement {
            transform.translation = destination;
            commands.entity(entity).remove::<MovingTo>();
        } else {
            transform.translation += direction.normalize() * movement;
        }
    }
}

fn wrap_left_right(mut query: Query<(&mut Transform, Option<&mut MovingTo>)>) {
    const MARGIN: f32 = 2.0 * GRID_SIZE;

    for (mut transform, moving_to) in &mut query {
        let new_x = (transform.translation.x + MARGIN).rem_euclid(WIDTH + MARGIN * 2.0) - MARGIN;
        // Check so we don't trigger change detection
        if (transform.translation.x - new_x).abs() > MARGIN {
            if let Some(mut moving_to) = moving_to {
                moving_to.x += new_x - transform.translation.x;
            }
            transform.translation.x = new_x;
        }
    }
}
