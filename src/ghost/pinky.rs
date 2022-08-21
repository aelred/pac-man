use bevy::prelude::*;

use crate::{grid::GridLocation, level::HEIGHT_TILES, movement::Dir, player::Player};

use super::{GhostSpawner, Mode, Personality, Target};

#[derive(Component, Default)]
pub struct Pinky;

impl Personality for Pinky {
    const NAME: &'static str = "Pinky";
    const COLOR: Color = Color::rgb(1.0, 0.72, 1.0);
    const SCATTER: GridLocation = GridLocation {
        x: 2,
        y: HEIGHT_TILES as isize - 1,
    };

    fn get_atlas(assets: &GhostSpawner) -> &Handle<TextureAtlas> {
        &assets.pinky
    }
}

pub fn chase(
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