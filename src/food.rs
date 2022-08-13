use crate::grid::GridLocation;
use crate::player::PlayerMovement;
use crate::score::{Score, UpdateScore};
use bevy::prelude::*;

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(eat.label(UpdateScore).before(PlayerMovement));
    }
}

#[derive(Component)]
pub struct Food {
    pub points: u32,
}

#[derive(Component)]
pub struct Eater;

fn eat(
    mut commands: Commands,
    mut score: ResMut<Score>,
    foods: Query<(Entity, &Food, &GridLocation)>,
    eater: Query<&GridLocation, With<Eater>>,
) {
    for eater_location in &eater {
        for (food_entity, food, food_location) in &foods {
            if eater_location == food_location {
                commands.entity(food_entity).despawn();
                *score += food.points;
            }
        }
    }
}
