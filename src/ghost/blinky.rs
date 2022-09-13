use bevy::prelude::*;

use crate::{
    grid::GridLocation,
    level::{HEIGHT_TILES, WIDTH_TILES},
    player::Player,
};

use super::{ActiveGhost, Ghost, Mode, Personality, Target};

#[derive(Component, Default)]
pub struct Blinky;

impl Personality for Blinky {
    const NAME: &'static str = "Blinky";
    const COLOR: Color = Color::RED;
    const SCATTER: GridLocation = GridLocation {
        x: WIDTH_TILES as isize - 3,
        y: HEIGHT_TILES as isize - 1,
    };
    const GHOST: Ghost = Ghost::Blinky;
}

pub fn chase(
    mode: Res<Mode>,
    mut query: Query<&mut Target, (ActiveGhost, With<Blinky>)>,
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
