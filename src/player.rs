use crate::grid::{GridLocation, GridMoving};
use crate::layout::Layout;
use crate::movement::{Collides, Dir};
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Lives>()
            .add_event::<PlayerDied>()
            .add_system(player_controls)
            .add_system(move_player.label(PlayerMovement).after(player_controls))
            .add_system(die_when_touching_deadly);
    }
}

#[derive(Clone, SystemLabel)]
pub struct PlayerMovement;

#[derive(Component)]
pub struct Deadly;

#[derive(Component, Default)]
pub struct Player {
    pub next_dir: Option<Dir>,
}

#[derive(Deref, DerefMut)]
pub struct Lives(usize);

impl Default for Lives {
    fn default() -> Self {
        Self(5)
    }
}

pub struct PlayerDied;

fn player_controls(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Player>) {
    let mut player = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        player.next_dir = Some(Dir::Left);
    }
    if keyboard_input.pressed(KeyCode::Right) {
        player.next_dir = Some(Dir::Right);
    }
    if keyboard_input.pressed(KeyCode::Down) {
        player.next_dir = Some(Dir::Down);
    }
    if keyboard_input.pressed(KeyCode::Up) {
        player.next_dir = Some(Dir::Up);
    }
}

fn move_player(
    layout: Res<Layout>,
    mut query: Query<(&GridLocation, &Collides, &Player, &mut Dir), Without<GridMoving>>,
) {
    for (location, collides, player, mut dir) in &mut query {
        if let Some(next) = player.next_dir {
            if !collides.at(&layout, &location.shift(next)) {
                *dir = next;
            }
        }
    }
}

fn die_when_touching_deadly(
    player: Query<&GridLocation, With<Player>>,
    deadlies: Query<&GridLocation, With<Deadly>>,
    mut death_events: EventWriter<PlayerDied>,
) {
    let player = player.single();

    for deadly in &deadlies {
        if player == deadly {
            death_events.send(PlayerDied);
            return;
        }
    }
}
