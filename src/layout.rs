use bevy::prelude::*;

use crate::{
    grid::GridLocation,
    level::{HEIGHT_TILES, WIDTH_TILES},
};

pub struct LayoutPlugin;

impl Plugin for LayoutPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Layout::load());
    }
}

pub struct Layout([[Option<Tile>; HEIGHT_TILES]; WIDTH_TILES]);

impl Layout {
    fn load() -> Layout {
        let mut tiles = [[None; HEIGHT_TILES]; WIDTH_TILES];

        let level_bmp = include_bytes!("../assets/level.bmp");

        // Find start of image data
        const OFFSET: usize = 0x0A;
        let offset = u32::from_le_bytes(level_bmp[OFFSET..OFFSET + 4].try_into().unwrap()) as usize;

        let mut img_data = level_bmp[offset..].iter();

        for y in 0..HEIGHT_TILES {
            for x in 0..WIDTH_TILES {
                let tile = match img_data.next().unwrap() {
                    0x01 => Some(Tile::Wall),
                    0x02 => Some(Tile::Dot),
                    0x04 => Some(Tile::Energizer),
                    0x05 => Some(Tile::PacMan),
                    0x06 => Some(Tile::Door),
                    0x07 => Some(Tile::Blinky),
                    0x08 => Some(Tile::Pinky),
                    0x09 => Some(Tile::Inky),
                    0x0A => Some(Tile::Clyde),
                    _ => None,
                };
                tiles[x][y] = tile;
            }
        }

        Layout(tiles)
    }

    pub fn get(&self, loc: &GridLocation) -> Option<Tile> {
        self.0[(loc.x.max(0) as usize).min(WIDTH_TILES - 1)][loc.y as usize]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Tile {
    Wall,
    Door,
    Dot,
    Energizer,
    PacMan,
    Blinky,
    Pinky,
    Inky,
    Clyde,
}
