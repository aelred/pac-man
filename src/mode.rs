use std::time::Duration;

use bevy::prelude::*;

pub struct ModePlugin;

impl Plugin for ModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Mode>()
            .init_resource::<ModeTimer>()
            .init_resource::<FrightenedTimer>()
            .add_system(tick_mode.label(TickMode).label(SetMode))
            .add_system(
                tick_frightened
                    .label(TickMode)
                    .label(SetMode)
                    .after(tick_mode),
            )
            .add_system(start_frightened_timer.after(SetMode));
    }
}

#[derive(SystemLabel)]
pub struct TickMode;

#[derive(SystemLabel)]
pub struct SetMode;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Scatter,
    Chase,
    Frightened,
}

impl Default for Mode {
    fn default() -> Self {
        MODE_TABLE[0].0
    }
}

struct ModeTimer {
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

#[derive(Deref, DerefMut)]
struct FrightenedTimer(Timer);

impl Default for FrightenedTimer {
    fn default() -> Self {
        let mut timer = Timer::new(Duration::from_secs_f32(6.0), false);
        timer.pause();
        FrightenedTimer(timer)
    }
}

fn tick_mode(time: Res<Time>, mut mode_timer: ResMut<ModeTimer>, mut mode: ResMut<Mode>) {
    if !mode_timer.timer.tick(time.delta()).finished() {
        return;
    }

    mode_timer.index += 1;
    let (new_mode, new_duration) = MODE_TABLE[mode_timer.index];
    info!("{:?} mode for {:?} seconds", new_mode, new_duration);
    *mode = new_mode;

    if new_duration.is_finite() {
        let new_duration = Duration::from_secs_f32(new_duration);
        mode_timer.timer.set_duration(new_duration);
    } else {
        mode_timer.timer.reset();
        mode_timer.timer.pause();
    }
}

fn start_frightened_timer(
    mut mode_timer: ResMut<ModeTimer>,
    mut frightened_timer: ResMut<FrightenedTimer>,
    mode: Res<Mode>,
) {
    if mode.is_changed() && *mode == Mode::Frightened {
        mode_timer.timer.pause();
        frightened_timer.reset();
        frightened_timer.unpause();
    }
}

fn tick_frightened(
    time: Res<Time>,
    mut mode_timer: ResMut<ModeTimer>,
    mut frightened_timer: ResMut<FrightenedTimer>,
    mut mode: ResMut<Mode>,
) {
    if frightened_timer.tick(time.delta()).finished() {
        mode_timer.timer.unpause();
        frightened_timer.pause();
        *mode = MODE_TABLE[mode_timer.index].0;
    }
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
