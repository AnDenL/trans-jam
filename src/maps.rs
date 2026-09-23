use macroquad::prelude::*;

pub const TILE_SIZE: f32 = 120.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Wall,
    SpawnP1,
    SpawnP2,
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn download_file_js(name_ptr: *const u8, name_len: usize, data_ptr: *const u8, data_len: usize);
}

pub fn save_file_crossplatform(filename: &str, content: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        download_file_js(
            filename.as_ptr(),
            filename.len(),
            content.as_ptr(),
            content.len(),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = std::fs::write(filename, content) {
            eprintln!("Error while writing file! {}: {}", filename, e);
        } else {
            println!("File succesfully downloaded: {}", filename);
        }
    }
}

pub fn load_default_map() -> Map {
    const DEFAULT_MAP_DATA: &str = include_str!("../map.txt");
    Map::from_string_repr(DEFAULT_MAP_DATA).unwrap_or_default()
}

#[derive(Clone, Debug)]
pub struct Map {
    pub w: usize,
    pub h: usize,
    pub tiles: Vec<Tile>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new(30, 20)
    }
}

impl Map {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            tiles: vec![Tile::Empty; w * h],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Tile {
        if x < 0 || x >= self.w as i32 || y < 0 || y >= self.h as i32 {
            return Tile::Wall;
        }
        self.tiles[y as usize * self.w + x as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, tile: Tile) {
        if x >= 0 && x < self.w as i32 && y >= 0 && y < self.h as i32 {
            self.tiles[y as usize * self.w + x as usize] = tile;
        }
    }

    pub fn resize(&mut self, new_w: usize, new_h: usize) {
        let mut new_tiles = vec![Tile::Empty; new_w * new_h];
        for y in 0..self.h.min(new_h) {
            for x in 0..self.w.min(new_w) {
                new_tiles[y * new_w + x] = self.tiles[y * self.w + x];
            }
        }
        self.w = new_w;
        self.h = new_h;
        self.tiles = new_tiles;
    }

    pub fn get_spawns(&self) -> (Vec<Vec2>, Vec<Vec2>) {
        let mut p1 = Vec::new();
        let mut p2 = Vec::new();

        for y in 0..self.h {
            for x in 0..self.w {
                let center = vec2(
                    x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                    y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                );
                match self.tiles[y * self.w + x] {
                    Tile::SpawnP1 => p1.push(center),
                    Tile::SpawnP2 => p2.push(center),
                    _ => {}
                }
            }
        }

        if p1.is_empty() {
            p1.push(vec2(TILE_SIZE * 1.5, TILE_SIZE * 1.5));
        }
        if p2.is_empty() {
            p2.push(vec2(
                (self.w as f32 - 1.5) * TILE_SIZE,
                (self.h as f32 - 1.5) * TILE_SIZE,
            ));
        }

        (p1, p2)
    }

    pub fn to_string_repr(&self) -> String {
        let mut s = format!("{} {}\n", self.w, self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                s.push(match self.tiles[y * self.w + x] {
                    Tile::Wall => '#',
                    Tile::Empty => '.',
                    Tile::SpawnP1 => '1',
                    Tile::SpawnP2 => '2',
                });
            }
            s.push('\n');
        }
        s
    }

    pub fn from_string_repr(content: &str) -> Option<Self> {
        let mut lines = content.lines();
        let header = lines.next()?;
        let dims: Vec<usize> = header
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if dims.len() != 2 {
            return None;
        }

        let (w, h) = (dims[0], dims[1]);
        let mut tiles = Vec::with_capacity(w * h);

        for line in lines {
            for ch in line.chars() {
                tiles.push(match ch {
                    '#' => Tile::Wall,
                    '1' => Tile::SpawnP1,
                    '2' => Tile::SpawnP2,
                    _ => Tile::Empty,
                });
            }
        }

        if tiles.len() == w * h {
            Some(Self { w, h, tiles })
        } else {
            None
        }
    }

    pub fn save(&self, path: &str) {
        let data = self.to_string_repr();
        save_file_crossplatform(path, &data);
    }

    pub fn load(path: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Some(map) = Self::from_string_repr(&content) {
                    return map;
                }
            }
        }
        println!("File '{}' not found, using default map.", path);
        load_default_map()
    }
}
