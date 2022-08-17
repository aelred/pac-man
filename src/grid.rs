use crate::movement::Dir;
use bevy::prelude::*;
use std::{ops::Mul, time::Duration};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_transform_from_grid_location)
            .add_system(update_transform_from_grid_moving)
            .add_system(move_on_grid.label(MoveOnGrid));
    }
}

#[derive(SystemLabel)]
pub struct MoveOnGrid;

#[derive(Component)]
pub struct Layer(pub f32);

impl Layer {
    pub const BACKGROUND: Self = Self(1.0);
    pub const FOREGROUND: Self = Self(5.0);
    pub const UI: Self = Self(9.0);
}

impl Default for Layer {
    fn default() -> Self {
        Self::FOREGROUND
    }
}

#[derive(Bundle, Default)]
pub struct GridBundle {
    pub grid: Grid,
    pub location: GridLocation,
    pub layer: Layer,
    #[bundle]
    pub _transform: TransformBundle,
}

#[derive(Component)]
pub struct Grid {
    pub size: Vec2,
    pub offset: Vec2,
}

impl Grid {
    pub fn to_vec2(&self, location: GridLocation) -> Vec2 {
        self.offset + self.size * location.to_unscaled_vec2()
    }

    pub fn center_offset(&self) -> Grid {
        Grid {
            size: self.size,
            offset: self.offset + self.size / 2.0,
        }
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            size: Vec2::new(1.0, 1.0),
            offset: Vec2::ZERO,
        }
    }
}

impl Mul<f32> for Grid {
    type Output = Grid;

    fn mul(self, rhs: f32) -> Self::Output {
        Grid {
            size: self.size * rhs,
            offset: self.offset,
        }
    }
}

#[derive(Component, Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct GridLocation {
    pub x: isize,
    pub y: isize,
}

impl GridLocation {
    pub fn shift(&self, dir: Dir) -> Self {
        let (dx, dy) = match dir {
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
            Dir::Down => (0, -1),
            Dir::Up => (0, 1),
        };
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub fn to_unscaled_vec2(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct GridMoving {
    pub destination: GridLocation,
    pub progress: f32,
    pub duration: Duration,
}

impl Default for GridMoving {
    fn default() -> Self {
        GridMoving {
            destination: Default::default(),
            progress: 0.0,
            duration: Duration::from_secs(1),
        }
    }
}

fn update_transform_from_grid_location(
    mut query: Query<(&Grid, &GridLocation, &Layer, &mut Transform), Without<GridMoving>>,
) {
    for (grid, location, layer, mut transform) in query.iter_mut() {
        set_transform_from_unscaled_grid(&mut transform, grid, layer, location.to_unscaled_vec2());
    }
}

fn update_transform_from_grid_moving(
    mut query: Query<(&Grid, &GridLocation, &GridMoving, &Layer, &mut Transform)>,
) {
    for (grid, location, moving, layer, mut transform) in query.iter_mut() {
        let unscaled = location.to_unscaled_vec2() * (1.0 - moving.progress)
            + moving.destination.to_unscaled_vec2() * moving.progress;
        set_transform_from_unscaled_grid(&mut transform, grid, layer, unscaled);
    }
}

fn set_transform_from_unscaled_grid(
    transform: &mut Transform,
    grid: &Grid,
    layer: &Layer,
    vec: Vec2,
) {
    transform.translation = (grid.offset + grid.size * vec).extend(layer.0);
}

fn move_on_grid(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut GridLocation, &mut GridMoving)>,
) {
    for (entity, mut location, mut moving) in query.iter_mut() {
        // When `Duration / Duration` is stable we can simplify this
        moving.progress += time.delta_seconds() / moving.duration.as_secs_f32();

        if moving.progress >= 1.0 {
            *location = moving.destination;
            commands.entity(entity).remove::<GridMoving>();
        }
    }
}
