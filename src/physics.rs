use crate::{
    PLAYER_RADIUS,
    maps::{Map, TILE_SIZE, Tile},
};
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

        if map_x < 0 || map_x >= map.w as i32 || map_y < 0 || map_y >= map.h as i32 {
            hit = true;
            hit_tile = Tile::Wall;
            break;
        }

        let tile = map.get(map_x, map_y);
        if tile == Tile::Wall || tile == Tile::BreakableWall || tile == Tile::Mirror {
            hit = true;
            hit_tile = tile;
            break;
        }
    }

    if hit && dist <= max_dist {
        Some(TileHit {
            point: origin + dir * dist,
            normal: hit_normal,
            distance: dist,
            tile_coord: (map_x as usize, map_y as usize),
            tile_type: hit_tile,
        })
    } else {
        None
    }
}

pub fn check_circle_hit(
    origin: Vec2,
    dir: Vec2,
    max_dist: f32,
    circle_pos: Vec2,
    radius: f32,
) -> Option<f32> {
    let to_circle = circle_pos - origin;
    if to_circle.length_squared() <= radius * radius {
        return Some(0.0);
    }
    let proj = to_circle.dot(dir);
    if proj <= 0.0 || proj >= max_dist {
        return None;
    }
    let perp_sq = to_circle.length_squared() - proj * proj;
    let hit_rad = radius + 6.0;
    if perp_sq <= hit_rad * hit_rad {
        Some(proj)
    } else {
        None
    }
}

pub fn is_circle_colliding(pos: Vec2, radius: f32, map: &Map, ignore_breakable: bool) -> bool {
    let min_tx = ((pos.x - radius) / TILE_SIZE).floor() as i32;
    let max_tx = ((pos.x + radius) / TILE_SIZE).floor() as i32;
    let min_ty = ((pos.y - radius) / TILE_SIZE).floor() as i32;
    let max_ty = ((pos.y + radius) / TILE_SIZE).floor() as i32;

    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            let tile = map.get(tx, ty);
            let is_solid = match tile {
                Tile::Wall | Tile::Mirror | Tile::Glass => true,
                Tile::BreakableWall => !ignore_breakable,
                _ => false,
            };

            if is_solid {
                let tile_min_x = tx as f32 * TILE_SIZE;
                let tile_max_x = (tx + 1) as f32 * TILE_SIZE;
                let tile_min_y = ty as f32 * TILE_SIZE;
                let tile_max_y = (ty + 1) as f32 * TILE_SIZE;

                let closest_x = pos.x.clamp(tile_min_x, tile_max_x);
                let closest_y = pos.y.clamp(tile_min_y, tile_max_y);

                let dx = pos.x - closest_x;
                let dy = pos.y - closest_y;

                if dx * dx + dy * dy < (radius - 0.15) * (radius - 0.15) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn resolve_circle_collision(pos: &mut Vec2, radius: f32, map: &Map, ignore_breakable: bool) {
    for _ in 0..4 {
        let min_tx = ((pos.x - radius) / TILE_SIZE).floor() as i32;
        let max_tx = ((pos.x + radius) / TILE_SIZE).floor() as i32;
        let min_ty = ((pos.y - radius) / TILE_SIZE).floor() as i32;
        let max_ty = ((pos.y + radius) / TILE_SIZE).floor() as i32;

        let mut max_pen = 0.0f32;
        let mut best_push = Vec2::ZERO;

        for ty in min_ty..=max_ty {
            for tx in min_tx..=max_tx {
                let tile = map.get(tx, ty);
                let is_solid = match tile {
                    Tile::Wall | Tile::Mirror | Tile::Glass => true,
                    Tile::BreakableWall => !ignore_breakable,
                    _ => false,
                };

                if is_solid {
                    let tile_min = vec2(tx as f32 * TILE_SIZE, ty as f32 * TILE_SIZE);
                    let tile_max = tile_min + vec2(TILE_SIZE, TILE_SIZE);

                    let closest = vec2(
                        pos.x.clamp(tile_min.x, tile_max.x),
                        pos.y.clamp(tile_min.y, tile_max.y),
                    );

                    let diff = *pos - closest;
                    let dist_sq = diff.length_squared();

                    if dist_sq < radius * radius {
                        let dist = dist_sq.sqrt();
                        let (normal, pen) = if dist > 0.0001 {
                            (diff / dist, radius - dist)
                        } else {
                            let dl = pos.x - tile_min.x;
                            let dr = tile_max.x - pos.x;
                            let db = pos.y - tile_min.y;
                            let dt = tile_max.y - pos.y;
                            let min_d = dl.min(dr).min(db).min(dt);
                            if min_d == dl {
                                (vec2(-1.0, 0.0), radius + dl)
                            } else if min_d == dr {
                                (vec2(1.0, 0.0), radius + dr)
                            } else if min_d == db {
                                (vec2(0.0, -1.0), radius + db)
                            } else {
                                (vec2(0.0, 1.0), radius + dt)
                            }
                        };

                        if pen > max_pen {
                            max_pen = pen;
                            best_push = normal * pen;
                        }
                    }
                }
            }
        }

        if max_pen > 0.001 {
            *pos += best_push * 1.03;
        } else {
            break;
        }
    }
}

pub enum ZipperResult {
    Teleported(Vec2),
    FellIntoVoid,
    Failed,
}

pub fn calculate_zipper(origin: Vec2, dir: Vec2, map: &Map) -> ZipperResult {
    if dir.length_squared() < 0.0001 {
        return ZipperResult::Failed;
    }
    let dir = dir.normalize();
    let world_w = map.w as f32 * TILE_SIZE;
    let world_h = map.h as f32 * TILE_SIZE;

    let wall_hit = match raycast_grid(origin, dir, 400.0, map) {
        Some(hit) => hit,
        None => return ZipperResult::Failed,
    };

    let step_size = 24.0;
    let mut current_pos = wall_hit.point + dir * (PLAYER_RADIUS + 10.0);

    for _ in 0..32 {
        if current_pos.x <= PLAYER_RADIUS
            || current_pos.x >= world_w - PLAYER_RADIUS
            || current_pos.y <= PLAYER_RADIUS
            || current_pos.y >= world_h - PLAYER_RADIUS
        {
            return ZipperResult::FellIntoVoid;
        }

        if !is_circle_colliding(current_pos, PLAYER_RADIUS, map, false) {
            return ZipperResult::Teleported(current_pos);
        }

        current_pos += dir * step_size;
    }

    ZipperResult::Failed
}
