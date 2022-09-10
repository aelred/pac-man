use crate::movement::Dir;
use bevy::prelude::*;
use std::ops::Mul;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_transform_from_grid_location
                .in_ambiguity_set(UpdateTransform)
                .after(SetGridLocation),
        )
        .add_system(
            update_transform_from_grid_moving
                .in_ambiguity_set(UpdateTransform)
                .after(SetGridMoving),
        )
        .add_system(move_on_grid.label(SetGridLocation).label(SetGridMoving));
    }
}

#[derive(AmbiguitySetLabel)]
struct UpdateTransform;

#[derive(SystemLabel)]
pub struct SetGridLocation;

#[derive(SystemLabel)]
pub struct SetGridMoving;

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
        self.shift_by(dir, 1)
    }

    pub fn shift_by(&self, dir: Dir, amount: isize) -> Self {
        let (dx, dy) = match dir {
            Dir::Left => (-amount, 0),
            Dir::Right => (amount, 0),
            Dir::Down => (0, -amount),
            Dir::Up => (0, amount),
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

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct GridMoving {
    pub destination: GridLocation,
    pub progress: f32,
}

// Speed in pixels per second
#[derive(Component, Default, Deref)]
pub struct Speed(pub f32);

impl Mul<f32> for Speed {
    type Output = Speed;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
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
    mut query: Query<(Entity, &Grid, &Speed, &mut GridLocation, &mut GridMoving)>,
) {
    for (entity, grid, speed, mut location, mut moving) in query.iter_mut() {
        // TODO: maybe this should be reworked to have movement happen outside of the grid
        let distance = (grid.to_vec2(moving.destination) - grid.to_vec2(*location)).length();

        moving.progress += **speed * time.delta_seconds() / distance;

        if moving.progress >= 1.0 {
            *location = moving.destination;
            commands.entity(entity).remove::<GridMoving>();
        }
    }
}
