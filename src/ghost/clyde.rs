use bevy::prelude::*;

use crate::{grid::GridLocation, player::Player};

use super::{Ghost, Mode, Personality, Target};

#[derive(Component, Default)]
pub struct Clyde;

impl Personality for Clyde {
    const NAME: &'static str = "Clyde";
    const COLOR: Color = Color::rgb(1.0, 0.72, 0.32);
    const SCATTER: GridLocation = GridLocation { x: 0, y: 0 };
    const GHOST: Ghost = Ghost::Clyde;
}

pub fn chase(
    mode: Res<Mode>,
    mut query: Query<(&GridLocation, &mut Target), With<Clyde>>,
    player: Query<&GridLocation, With<Player>>,
) {
    if *mode != Mode::Chase {
        return;
    }

    for (clyde_loc, mut target) in &mut query {
        let player_loc = player.get_single().unwrap();

        let dist = player_loc
            .to_unscaled_vec2()
            .distance_squared(clyde_loc.to_unscaled_vec2());

        let new_target = if dist > 8.0 * 8.0 {
            *player_loc
        } else {
            Clyde::SCATTER
        };

        if **target != new_target {
            **target = new_target;
        }
    }
}
