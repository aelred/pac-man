use crate::ghost::{Frightened, Ghost, Respawning};
use crate::grid::{GridLocation, Speed};
use crate::mode::Mode;
use crate::movement::{Dir, NextDir, SetNextDir, BASE_SPEED};
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Lives>()
            .add_event::<PlayerDied>()
            .add_system(player_controls.label(SetNextDir).before(PlayerDeath))
            .add_system(
                die_when_touching_ghost
                    .label(PlayerDeath)
                    .in_ambiguity_set(PlayerDeath),
            )
            .add_system(lose_life_when_dying.label(UpdateLives).after(PlayerDeath))
            .add_system(set_speed);
    }
}

#[derive(AmbiguitySetLabel, SystemLabel)]
pub struct PlayerDeath;

#[derive(SystemLabel)]
pub struct UpdateLives;

#[derive(Component)]
pub struct Player;

#[derive(Deref, DerefMut)]
pub struct Lives(usize);

impl Default for Lives {
    fn default() -> Self {
        Self(5)
    }
}

pub struct PlayerDied;

fn player_controls(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut NextDir, With<Player>>,
) {
    let mut player = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        **player = Some(Dir::Left);
    }
    if keyboard_input.pressed(KeyCode::Right) {
        **player = Some(Dir::Right);
    }
    if keyboard_input.pressed(KeyCode::Down) {
        **player = Some(Dir::Down);
    }
    if keyboard_input.pressed(KeyCode::Up) {
        **player = Some(Dir::Up);
    }
}

fn die_when_touching_ghost(
    player: Query<&GridLocation, With<Player>>,
    ghosts: Query<&GridLocation, (With<Ghost>, Without<Frightened>, Without<Respawning>)>,
    mut death_events: EventWriter<PlayerDied>,
) {
    let player = player.single();

    for ghost in &ghosts {
        if player == ghost {
            death_events.send(PlayerDied);
            return;
        }
    }
}

fn lose_life_when_dying(mut deaths: EventReader<PlayerDied>, mut lives: ResMut<Lives>) {
    for _ in deaths.iter() {
        // saturating_sub until we have game over behaviour
        **lives = lives.saturating_sub(1);
    }
}

fn set_speed(mode: Res<Mode>, mut player_speed: Query<&mut Speed, With<Player>>) {
    if !mode.is_changed() {
        return;
    }

    let new_speed = if let Mode::Frightened = *mode {
        0.9
    } else {
        0.8
    };

    *player_speed.single_mut() = BASE_SPEED * new_speed
}
