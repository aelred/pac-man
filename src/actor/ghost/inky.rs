use bevy::prelude::*;

use crate::{actor::movement::Dir, actor::player::Player, grid::GridLocation, level::WIDTH_TILES};

use super::{blinky::Blinky, ActiveGhost, Mode, Personality, PersonalityT, Target};

#[derive(Component, Default)]
pub struct Inky;

impl PersonalityT for Inky {
    const NAME: &'static str = "Inky";
    const COLOR: Color = Color::CYAN;
    const SCATTER: GridLocation = GridLocation {
        x: WIDTH_TILES as isize - 1,
        y: 0,
    };
    const VALUE: Personality = Personality::Inky;
}

pub fn chase(
    mode: Res<Mode>,
    mut query: Query<&mut Target, (ActiveGhost, With<Inky>)>,
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
