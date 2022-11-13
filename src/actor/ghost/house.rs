use bevy::prelude::*;

#[derive(Component, Default)]
pub struct InHouse;

/// Each ghost has a "dot counter" that increments when the player eat dots
/// while they're in the house, until it reaches a limit and they leave.
/// The counter is retained even when the ghost respawns.
#[derive(Component, Default)]
pub struct DotCounter(u8);
