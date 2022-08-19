use std::time::Duration;

use bevy::prelude::*;

use crate::{
    grid::{GridLocation, MoveOnGrid},
    layout::Layout,
    level::{HEIGHT_TILES, WIDTH_TILES},
    movement::{Collides, Dir, NextDir, SetDir},
    player::Player,
};

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Mode>()
            .init_resource::<ModeTimer>()
            .add_system(choose_next_dir.after(SetDir).before(MoveOnGrid))
            .add_system(tick_mode)
            .add_system(scatter)
            .add_system(blinky_chase)
            .add_system(pinky_chase)
            .add_system(inky_chase)
            .add_system(clyde_chase);
    }
}

#[derive(Bundle, Default)]
pub struct GhostBundle {
    pub scatter_target: ScatterTarget,
    pub _target: Target,
}

const MODE_TABLE: [(Mode, f32); 8] = [
    (Mode::Scatter, 7.0),
    (Mode::Chase, 20.0),
    (Mode::Scatter, 7.0),
    (Mode::Chase, 20.0),
    (Mode::Scatter, 5.0),
    (Mode::Chase, 20.0),
    (Mode::Scatter, 5.0),
    (Mode::Chase, f32::INFINITY),
];

pub trait Ghost: Component + Default {
    const NAME: &'static str;
    const SCATTER: GridLocation;
}

#[derive(Component, Default)]
pub struct Blinky;

impl Ghost for Blinky {
    const NAME: &'static str = "Blinky";
    const SCATTER: GridLocation = GridLocation {
        x: WIDTH_TILES as isize - 3,
        y: HEIGHT_TILES as isize - 1,
    };
}

#[derive(Component, Default)]
pub struct Pinky;

impl Ghost for Pinky {
    const NAME: &'static str = "Pinky";
    const SCATTER: GridLocation = GridLocation {
        x: 2,
        y: HEIGHT_TILES as isize - 1,
    };
}

#[derive(Component, Default)]
pub struct Inky;

impl Ghost for Inky {
    const NAME: &'static str = "Inky";
    const SCATTER: GridLocation = GridLocation { x: 0, y: 0 };
}

#[derive(Component, Default)]
pub struct Clyde;

impl Ghost for Clyde {
    const NAME: &'static str = "Clyde";
    const SCATTER: GridLocation = GridLocation {
        x: WIDTH_TILES as isize - 1,
        y: 0,
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    Scatter,
    Chase,
}

impl Default for Mode {
    fn default() -> Self {
        MODE_TABLE[0].0
    }
}

pub struct ModeTimer {
    index: usize,
    timer: Timer,
}

impl Default for ModeTimer {
    fn default() -> Self {
        Self {
            index: 0,
            timer: Timer::new(Duration::from_secs_f32(MODE_TABLE[0].1), true),
        }
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct Target(pub GridLocation);

#[derive(Component, Default, Deref, DerefMut)]
pub struct ScatterTarget(pub GridLocation);

fn tick_mode(time: Res<Time>, mut mode_timer: ResMut<ModeTimer>, mut mode: ResMut<Mode>) {
    mode_timer.timer.tick(time.delta());

    if mode_timer.timer.finished() {
        mode_timer.index += 1;
        let (new_mode, new_duration) = MODE_TABLE[mode_timer.index];
        info!("{:?} mode for {:?} seconds", new_mode, new_duration);
        *mode = new_mode;
        let new_duration = Duration::from_secs_f32(new_duration);
        mode_timer.timer.set_duration(new_duration);
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

fn blinky_chase(
    mode: Res<Mode>,
    mut query: Query<&mut Target, With<Blinky>>,
    player: Query<&GridLocation, With<Player>>,
) {
    if *mode != Mode::Chase {
        return;
    }

    for mut target in &mut query {
        let new_target = *player.get_single().unwrap();

        if **target != new_target {
            **target = new_target;
        }
    }
}

fn pinky_chase(
    mode: Res<Mode>,
    mut query: Query<&mut Target, With<Pinky>>,
    player: Query<(&GridLocation, &Dir), With<Player>>,
) {
    if *mode != Mode::Chase {
        return;
    }

    for mut target in &mut query {
        let (player_loc, player_dir) = player.get_single().unwrap();
        let new_target = player_loc.shift_by(*player_dir, 4);

        if **target != new_target {
            **target = new_target;
        }
    }
}

fn inky_chase(
    mode: Res<Mode>,
    mut query: Query<&mut Target, With<Inky>>,
    player: Query<(&GridLocation, &Dir), With<Player>>,
    blinky: Query<&GridLocation, With<Blinky>>,
) {
    if *mode != Mode::Chase {
        return;
    }

    for mut target in &mut query {
        let (player_loc, player_dir) = player.get_single().unwrap();
        let blinky_loc = blinky.get_single().unwrap();

        let mut new_target = player_loc.shift_by(*player_dir, 2);
        new_target.x += new_target.x - blinky_loc.x;
        new_target.y += new_target.y - blinky_loc.y;

        if **target != new_target {
            **target = new_target;
        }
    }
}

fn clyde_chase(
    mode: Res<Mode>,
    mut query: Query<(&GridLocation, &ScatterTarget, &mut Target), With<Clyde>>,
    player: Query<&GridLocation, With<Player>>,
) {
    if *mode != Mode::Chase {
        return;
    }

    for (clyde_loc, scatter_target, mut target) in &mut query {
        let player_loc = player.get_single().unwrap();

        let dist = player_loc
            .to_unscaled_vec2()
            .distance_squared(clyde_loc.to_unscaled_vec2());

        let new_target = if dist > 8.0 * 8.0 {
            player_loc
        } else {
            scatter_target
        };

        if **target != *new_target {
            **target = *new_target;
        }
    }
}

// Ghosts decide on their next direction one grid location BEFORE
fn choose_next_dir(
    layout: Res<Layout>,
    mut query: Query<
        (&Dir, &mut NextDir, &GridLocation, &Target, &Collides),
        Changed<GridLocation>,
    >,
) {
    for (dir, mut next_dir, loc, target, collides) in &mut query {
        let next_loc = loc.shift(*dir);

        if collides.at(&layout, &next_loc) {
            continue;
        }

        let mut distance_to_target = f32::MAX;

        for candidate_dir in &[Dir::Up, Dir::Left, Dir::Down, Dir::Right] {
            let candidate_loc = next_loc.shift(*candidate_dir);
            let collision = collides.at(&layout, &candidate_loc);

            let distance = candidate_loc
                .to_unscaled_vec2()
                .distance_squared(target.to_unscaled_vec2());

            if !collision && candidate_loc != *loc && distance < distance_to_target {
                **next_dir = Some(*candidate_dir);
                distance_to_target = distance;
            }
        }
    }
}
