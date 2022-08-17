use bevy::prelude::*;

use crate::{
    grid::{GridLocation, MoveOnGrid},
    layout::Layout,
    movement::{Collides, Dir, NextDir, SetDir},
};

pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(choose_next_dir.after(SetDir).before(MoveOnGrid));
    }
}

#[derive(Bundle, Default)]
pub struct GhostBundle {
    target: Target,
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct Target(pub GridLocation);

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
