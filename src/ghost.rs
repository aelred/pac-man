use std::time::Duration;

use bevy::prelude::*;

use crate::{
    grid::{GridLocation, MoveOnGrid},
    layout::Layout,
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
            .add_system(choose_target);
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

#[derive(Copy, Clone, Debug)]
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
        *mode = new_mode;
        let new_duration = Duration::from_secs_f32(new_duration);
        mode_timer.timer.set_duration(new_duration);
    }
}

fn choose_target(
    mode: Res<Mode>,
    mut query: Query<(&ScatterTarget, &mut Target)>,
    player: Query<&GridLocation, With<Player>>,
) {
    for (scatter_target, mut target) in &mut query {
        let new_target = match *mode {
            Mode::Scatter => **scatter_target,
            Mode::Chase => *player.get_single().unwrap(),
        };

        if **target != new_target {
            **target = new_target;
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
