use crate::grid::{GridLocation, SetGridLocation};
use crate::mode::{Mode, SetMode, TickMode};
use crate::score::{Score, UpdateScore};
use bevy::prelude::*;

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Eat>()
            .add_system(eat.after(SetGridLocation))
            .add_system(add_score.after(eat).label(UpdateScore))
            .add_system(eat_energizer.label(SetMode).after(eat).after(TickMode));
    }
}

#[derive(Component)]
pub struct Food {
    pub points: u32,
}

#[derive(Component)]
pub struct Eater;

#[derive(Component)]
pub struct Energizer;

struct Eat(Entity);

fn eat(
    mut commands: Commands,
    foods: Query<(Entity, &GridLocation), With<Food>>,
    eater: Query<&GridLocation, With<Eater>>,
    mut eat_events: EventWriter<Eat>,
) {
    for eater_location in &eater {
        for (food_entity, food_location) in &foods {
            if eater_location == food_location {
                eat_events.send(Eat(food_entity));
                commands.entity(food_entity).despawn();
            }
        }
    }
}

fn add_score(mut eat_events: EventReader<Eat>, mut score: ResMut<Score>, query: Query<&Food>) {
    for Eat(food) in eat_events.iter() {
        *score += query.get(*food).unwrap().points;
    }
}

fn eat_energizer(
    mut eat_events: EventReader<Eat>,
    energizers: Query<&Energizer>,
    mut mode: ResMut<Mode>,
) {
    for Eat(eaten) in eat_events.iter() {
        if energizers.contains(*eaten) {
            *mode = Mode::Frightened;
        }
    }
}
