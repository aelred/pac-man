use bevy::prelude::*;

use crate::{grid::GridLocation, level::WIDTH_TILES, movement::Dir, player::Player};

use super::{blinky::Blinky, GhostSpawner, Mode, Personality, Target};

#[derive(Component, Default)]
pub struct Inky;

impl Personality for Inky {
    const NAME: &'static str = "Inky";
    const COLOR: Color = Color::CYAN;
    const SCATTER: GridLocation = GridLocation {
        x: WIDTH_TILES as isize - 1,
        y: 0,
    };

    fn get_atlas(assets: &GhostSpawner) -> &Handle<TextureAtlas> {
        &assets.inky
    }
}

pub fn chase(
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
