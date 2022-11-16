use bevy::prelude::*;
use std::fmt;
use std::fmt::Formatter;
use std::ops::AddAssign;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Score(0))
            .insert_resource(HighScore(0))
            .add_system(update_high_score.label(UpdateHighScore).after(UpdateScore));
    }
}

#[derive(SystemLabel)]
pub struct UpdateScore;

#[derive(SystemLabel)]
pub struct UpdateHighScore;

#[derive(Resource)]
pub struct Score(u32);

impl fmt::Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AddAssign<u32> for Score {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

#[derive(Resource)]
pub struct HighScore(u32);

impl fmt::Display for HighScore {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

fn update_high_score(score: Res<Score>, mut high_score: ResMut<HighScore>) {
    if score.is_changed() && score.0 > high_score.0 {
        high_score.0 = score.0;
    }
}
