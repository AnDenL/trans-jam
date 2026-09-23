use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tile {
    Empty,
    Wall,
    Breakable,
    Glass,
    Pit,
}

impl Tile {
    pub fn blocks_movement(self) -> bool {
        matches!(self, Tile::Wall | Tile::Breakable | Tile::Glass | Tile::Pit)
    }

    pub fn blocks_ray(self) -> bool {
        matches!(self, Tile::Wall | Tile::Breakable | Tile::Glass)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Map {
    pub w: usize,
    pub h: usize,
    pub tiles: Vec<Tile>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            w: 10,
            h: 10,
            tiles: vec![Tile::Empty; 100],
        }
    }
}

impl Map {
    pub fn get_tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || x >= self.w as i32 || y < 0 || y >= self.h as i32 {
            return Tile::Wall;
        }
        self.tiles[y as usize * self.w + x as usize]
    }
}
