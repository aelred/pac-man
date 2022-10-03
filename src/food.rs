use crate::actor::ghost::Ghost;
use crate::actor::mode::{FrightenedMode, SetMode, TickMode};
use crate::actor::player::Player;
use crate::grid::GridLocation;
use crate::score::{Score, UpdateScore};
use bevy::prelude::*;

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Eat>()
            // To prevent cycles, eating happens the frame after pac-man moves there
            .add_system(eat.label(WriteEatEvent).in_ambiguity_set(WriteEatEvent))
            .add_system(add_score.after(WriteEatEvent).label(UpdateScore))
            .add_system(
                eat_energizer
                    .label(SetMode)
                    .after(WriteEatEvent)
                    .after(TickMode),
            )
            .add_system(destroy.after(WriteEatEvent));
    }
}

#[derive(SystemLabel, AmbiguitySetLabel)]
pub struct WriteEatEvent;

#[derive(Component, Default)]
pub struct Food {
    pub points: u32,
}

#[derive(Component)]
pub struct Energizer;

pub struct Eat(pub Entity);

fn eat(
    foods: Query<(Entity, &GridLocation), With<Food>>,
    player: Query<&GridLocation, With<Player>>,
    mut eat_events: EventWriter<Eat>,
) {
    for player_location in &player {
        for (food_entity, food_location) in &foods {
            if player_location == food_location {
                eat_events.send(Eat(food_entity));
            }
        }
    }
}

fn destroy(mut commands: Commands, mut eat_events: EventReader<Eat>, ghosts: Query<&Ghost>) {
    for Eat(food) in eat_events.iter() {
        if !ghosts.contains(*food) {
            commands.entity(*food).despawn();
        }
    }
}

fn add_score(mut eat_events: EventReader<Eat>, mut score: ResMut<Score>, query: Query<&Food>) {
    for Eat(food) in eat_events.iter() {
        *score += query.get(*food).expect("Eaten thing is not Food").points;
    }
}

fn eat_energizer(
    mut eat_events: EventReader<Eat>,
    energizers: Query<&Energizer>,
    mut mode: ResMut<FrightenedMode>,
) {
    for Eat(eaten) in eat_events.iter() {
        if energizers.contains(*eaten) {
            *mode = FrightenedMode::Enabled;
        }
    }
}
