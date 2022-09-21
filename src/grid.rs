use crate::actor::movement::Dir;
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use std::ops::Mul;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_grid_location_from_transform
                .after(SetTransform)
                .label(SetGridLocation),
        );
    }
}

#[derive(SystemLabel)]
pub struct SetTransform;

#[derive(SystemLabel)]
pub struct SetGridLocation;

#[derive(SystemLabel)]
pub struct SetGridMoving;

#[derive(Component, Copy, Clone)]
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
    grid: Grid,
    location: GridLocation,
    layer: Layer,
    #[bundle]
    transform: TransformBundle,
}

impl GridBundle {
    pub fn new(grid: Grid, location: GridLocation, layer: Layer) -> Self {
        Self {
            grid,
            location,
            layer,
            transform: TransformBundle {
                local: Transform {
                    translation: grid.to_vec2(location).extend(layer.0),
                    ..default()
                },
                ..default()
            },
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct Grid {
    pub size: Vec2,
    pub offset: Vec2,
}

impl Grid {
    pub fn to_vec2(&self, location: GridLocation) -> Vec2 {
        self.offset + self.size * location.to_unscaled_vec2()
    }

    pub fn to_grid_location(&self, vec: Vec2) -> GridLocation {
        let scaled = (vec - self.offset) / self.size;
        GridLocation {
            x: scaled.x.round() as isize,
            y: scaled.y.round() as isize,
        }
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

#[derive(Component, Debug, Copy, Clone, Default, Eq, PartialEq, Inspectable)]
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

#[derive(Component, Default, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub struct MovingTo(pub Vec3);

// Speed in pixels per second
#[derive(Component, Default, Deref)]
pub struct Speed(pub f32);

impl Mul<f32> for Speed {
    type Output = Speed;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

fn update_grid_location_from_transform(
    mut query: Query<(&Grid, &Transform, &mut GridLocation), Without<MovingTo>>,
) {
    for (grid, transform, mut location) in query.iter_mut() {
        let new_location = grid.to_grid_location(transform.translation.truncate());
        if *location != new_location {
            *location = new_location;
        }
    }
}
