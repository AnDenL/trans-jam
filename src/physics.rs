use crate::maps::{Map, Tile};
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct TileHit {
    pub point: Vec2,
    pub normal: Vec2,
    pub distance: f32,
    pub tile_coord: (usize, usize),
    pub tile_type: Tile,
}

pub fn raycast_grid(origin: Vec2, dir: Vec2, max_dist: f32, map: &Map) -> Option<TileHit> {
    if dir.length_squared() < 0.0001 {
        return None;
    }
    let dir = dir.normalize();

    let mut map_x = (origin.x / TILE_SIZE).floor() as i32;
    let mut map_y = (origin.y / TILE_SIZE).floor() as i32;

    let delta_dist_x = if dir.x.abs() < 1e-6 {
        f32::INFINITY
    } else {
        (TILE_SIZE / dir.x).abs()
    };
    let delta_dist_y = if dir.y.abs() < 1e-6 {
        f32::INFINITY
    } else {
        (TILE_SIZE / dir.y).abs()
    };

    let step_x = if dir.x < 0.0 { -1 } else { 1 };
    let step_y = if dir.y < 0.0 { -1 } else { 1 };

    let mut side_dist_x = if dir.x < 0.0 {
        (origin.x - map_x as f32 * TILE_SIZE) / dir.x.abs()
    } else {
        ((map_x + 1) as f32 * TILE_SIZE - origin.x) / dir.x.abs()
    };

    let mut side_dist_y = if dir.y < 0.0 {
        (origin.y - map_y as f32 * TILE_SIZE) / dir.y.abs()
    } else {
        ((map_y + 1) as f32 * TILE_SIZE - origin.y) / dir.y.abs()
    };

    let mut hit = false;
    let mut hit_normal = Vec2::ZERO;
    let mut dist = 0.0;
    let mut hit_tile = Tile::Empty;

    for _ in 0..64 {
        if side_dist_x < side_dist_y {
            dist = side_dist_x;
            side_dist_x += delta_dist_x;
            map_x += step_x;
            hit_normal = vec2(-step_x as f32, 0.0);
        } else {
            dist = side_dist_y;
            side_dist_y += delta_dist_y;
            map_y += step_y;
            hit_normal = vec2(0.0, -step_y as f32);
        }

        if dist > max_dist {
            break;
        }

        if map_x < 0 || map_x >= MAP_W as i32 || map_y < 0 || map_y >= MAP_H as i32 {
            hit = true;
            break;
        }

        let idx = map_y as usize * MAP_W + map_x as usize;
        if map[idx] {
            hit = true;
            break;
        }
    }

    if hit && dist <= max_dist {
        Some(TileHit {
            point: origin + dir * dist,
            normal: hit_normal,
            distance: dist,
            tile_coord: (map_x, map_y),
        })
    } else {
        None
    }
}
