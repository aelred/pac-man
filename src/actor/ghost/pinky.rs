use bevy::prelude::*;

use crate::{
    actor::mode::Mode, actor::movement::Dir, actor::player::Player, grid::GridLocation,
    level::HEIGHT_TILES,
};

use super::{ActiveGhost, Ghost, Personality, Target};

#[derive(Component, Default)]
pub struct Pinky;

impl Personality for Pinky {
    const NAME: &'static str = "Pinky";
    const COLOR: Color = Color::rgb(1.0, 0.72, 1.0);
    const SCATTER: GridLocation = GridLocation {
        x: 2,
        y: HEIGHT_TILES as isize - 1,
    };
    const GHOST: Ghost = Ghost::Pinky;
}

pub fn chase(
    mode: Res<Mode>,
    mut query: Query<&mut Target, (ActiveGhost, With<Pinky>)>,
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
