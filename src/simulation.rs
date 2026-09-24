use macroquad::prelude::*;

use crate::{
    PLAYER_RADIUS,
    maps::{Map, TILE_SIZE, Tile},
    physics::*,
    player::{Player, PlayerInput, kill_enemy, process_player},
};

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    type Input = PlayerInput;
    type InputPredictor = ggrs::PredictRepeatLast;
    type State = GameState;
    type Address = std::net::SocketAddr;
}

#[derive(Clone, Debug)]
pub enum GameEvent {
    Laser {
        from: Vec2,
        to: Vec2,
        color: Color,
    },
    Blink {
        from: Vec2,
        to: Vec2,
        color: Color,
    },
    Explosion {
        pos: Vec2,
        color: Color,
    },
    Shockwave {
        pos: Vec2,
        radius: f32,
        color: Color,
    },
    UsbDrop {
        pos: Vec2,
        color: Color,
    },
    UsbRecall {
        from: Vec2,
        to: Vec2,
        color: Color,
    },
    Zipper {
        from: Vec2,
        to: Vec2,
        color: Color,
    },
}

#[derive(Clone, Default)]
pub struct GameState {
    pub p1: Player,
    pub p2: Player,
    pub score_p1: u32,
    pub score_p2: u32,
    pub events: Vec<GameEvent>,
    pub map: Map,
    pub rng_seed: u64,
    pub frame_count: i32,
}

impl GameState {
    pub fn advance(&mut self, inputs: [(PlayerInput, ggrs::InputStatus); 2]) {
        self.events.clear();
        self.frame_count += 1;

        let (p1_spawns, p2_spawns) = self.map.get_spawns();

        let p1_spawn = p1_spawns[(self.score_p2 as usize) % p1_spawns.len()];
        let p2_spawn = p2_spawns[(self.score_p1 as usize) % p2_spawns.len()];

        process_player(
            &mut self.p1,
            &mut self.p2,
            &mut self.score_p1,
            &mut self.score_p2,
            inputs[0].0,
            &mut self.events,
            GOLD,
            SKYBLUE,
            p1_spawn,
            p2_spawn,
            &mut self.map,
            &mut self.rng_seed,
        );

        process_player(
            &mut self.p2,
            &mut self.p1,
            &mut self.score_p2,
            &mut self.score_p1,
            inputs[1].0,
            &mut self.events,
            SKYBLUE,
            GOLD,
            p2_spawn,
            p1_spawn,
            &mut self.map,
            &mut self.rng_seed,
        );
    }
}

pub fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    if x == 0 {
        x = 0xdeadbeef;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Weapon {
    #[default]
    Railgun,
    Shotgun,
    Piercing,
    WhiteOut,
    Stomp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Movement {
    #[default]
    Hook,
    Zipper,
    NailFile,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Utillity {
    #[default]
    None,
    Parry,
    ZaWarudo,
    Usb(Option<Vec2>),
}

pub fn roll_weapon(rng: &mut u64) -> Weapon {
    match xorshift64(rng) % 5 {
        0 => Weapon::Railgun,
        1 => Weapon::Shotgun,
        2 => Weapon::Piercing,
        3 => Weapon::WhiteOut,
        _ => Weapon::Stomp,
    }
}

pub fn roll_movement(rng: &mut u64) -> Movement {
    match xorshift64(rng) % 3 {
        0 => Movement::Zipper,
        1 => Movement::NailFile,
        _ => Movement::Hook,
    }
}

pub fn roll_util(rng: &mut u64) -> Utillity {
    match xorshift64(rng) % 3 {
        0 => Utillity::Usb(None),
        1 => Utillity::ZaWarudo,
        _ => Utillity::Parry,
    }
}

pub fn fire_ray(
    origin: Vec2,
    dir: Vec2,
    max_range: f32,
    p_active: &mut Player,
    p_enemy: &mut Player,
    score: &mut u32,
    enemy_score: &mut u32,
    my_spawn: Vec2,
    enemy_spawn: Vec2,
    events: &mut Vec<GameEvent>,
    my_color: Color,
    enemy_color: Color,
    map: &mut Map,
    rng: &mut u64,
) {
    let wall_hit = raycast_grid(origin, dir, max_range, map);
    let limit = wall_hit.map(|h| h.distance).unwrap_or(max_range);

    if let Some(hit_dist) = check_circle_hit(origin, dir, limit, p_enemy.pos, PLAYER_RADIUS) {
        if p_enemy.parry_timer > 0 {
            kill_enemy(p_active, enemy_score, my_spawn, events, my_color, rng);
            events.push(GameEvent::Laser {
                from: origin,
                to: p_enemy.pos,
                color: enemy_color,
            });
            events.push(GameEvent::Shockwave {
                pos: p_enemy.pos,
                radius: 160.0,
                color: GOLD,
            });
            events.push(GameEvent::Explosion {
                pos: p_enemy.pos,
                color: WHITE,
            });
            return;
        }

        kill_enemy(p_enemy, score, enemy_spawn, events, enemy_color, rng);
        events.push(GameEvent::Laser {
            from: origin,
            to: origin + dir * hit_dist,
            color: my_color,
        });
    } else {
        events.push(GameEvent::Laser {
            from: origin,
            to: origin + dir * limit,
            color: my_color,
        });

        if let Some(hit) = wall_hit {
            if hit.tile_type == Tile::BreakableWall {
                map.set(
                    hit.tile_coord.0 as i32,
                    hit.tile_coord.1 as i32,
                    Tile::Empty,
                );
                events.push(GameEvent::Explosion {
                    pos: hit.point,
                    color: WHITE,
                });
                events.push(GameEvent::Shockwave {
                    pos: hit.point,
                    radius: 80.0,
                    color: LIGHTGRAY,
                });
            }
        }
    }
}

pub fn execute_weapon(
    weapon: Weapon,
    p_active: &mut Player,
    p_enemy: &mut Player,
    score: &mut u32,
    enemy_score: &mut u32,
    my_spawn: Vec2,
    enemy_spawn: Vec2,
    events: &mut Vec<GameEvent>,
    my_color: Color,
    enemy_color: Color,
    map: &mut Map,
    rng: &mut u64,
) {
    let origin = p_active.pos;
    let dir = p_active.dir;

    match weapon {
        Weapon::Railgun => {
            fire_ray(
                origin,
                dir,
                3000.0,
                p_active,
                p_enemy,
                score,
                enemy_score,
                my_spawn,
                enemy_spawn,
                events,
                my_color,
                enemy_color,
                map,
                rng,
            );
        }

        Weapon::Shotgun => {
            let base_angle = dir.y.atan2(dir.x);
            let spreads = [-0.22, -0.11, 0.0, 0.11, 0.22];

            for offset in spreads {
                let angle = base_angle + offset;
                let spread_dir = vec2(angle.cos(), angle.sin());
                fire_ray(
                    origin,
                    spread_dir,
                    1000.0,
                    p_active,
                    p_enemy,
                    score,
                    enemy_score,
                    my_spawn,
                    enemy_spawn,
                    events,
                    my_color,
                    enemy_color,
                    map,
                    rng,
                );
            }
        }

        Weapon::Piercing => {
            let max_dist = 4000.0;

            if let Some(hit) = raycast_grid(origin, dir, max_dist, map) {
                events.push(GameEvent::Laser {
                    from: origin,
                    to: hit.point,
                    color: Color::new(0.05, 0.05, 0.3, 1.0),
                });

                if hit.tile_type == Tile::BreakableWall {
                    map.set(
                        hit.tile_coord.0 as i32,
                        hit.tile_coord.1 as i32,
                        Tile::Empty,
                    );
                    events.push(GameEvent::Explosion {
                        pos: hit.point,
                        color: WHITE,
                    });
                    events.push(GameEvent::Shockwave {
                        pos: hit.point,
                        radius: 80.0,
                        color: Color::new(1.0, 0.2, 0.8, 1.0),
                    });
                }

                if let Some(hit_dist) =
                    check_circle_hit(hit.point, dir, max_dist, p_enemy.pos, PLAYER_RADIUS)
                {
                    if p_enemy.parry_timer > 0 {
                        kill_enemy(p_active, enemy_score, my_spawn, events, my_color, rng);
                        events.push(GameEvent::Shockwave {
                            pos: p_enemy.pos,
                            radius: 160.0,
                            color: GOLD,
                        });
                        return;
                    }

                    kill_enemy(p_enemy, score, enemy_spawn, events, enemy_color, rng);
                    events.push(GameEvent::Laser {
                        from: hit.point,
                        to: hit.point + dir * hit_dist,
                        color: Color::new(1.0, 0.2, 0.8, 1.0),
                    });
                } else {
                    events.push(GameEvent::Laser {
                        from: hit.point,
                        to: hit.point + dir * max_dist,
                        color: Color::new(1.0, 0.2, 0.8, 1.0),
                    });
                }
            }
        }
        Weapon::WhiteOut => {
            let max_place_dist = 560.0;

            fire_ray(
                origin,
                dir,
                max_place_dist,
                p_active,
                p_enemy,
                score,
                enemy_score,
                my_spawn,
                enemy_spawn,
                events,
                my_color,
                enemy_color,
                map,
                rng,
            );

            let hit = raycast_grid(origin, dir, max_place_dist, map);
            let target_point = match hit {
                Some(h) => h.point - dir * (TILE_SIZE * 0.4),
                None => origin + dir * max_place_dist,
            };

            let tx = (target_point.x / TILE_SIZE).floor() as i32;
            let ty = (target_point.y / TILE_SIZE).floor() as i32;

            if map.get(tx, ty) == Tile::Empty {
                let tile_center = vec2(
                    tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                    ty as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                );

                if (p_enemy.pos - tile_center).length() < (PLAYER_RADIUS + TILE_SIZE * 0.45) {
                    kill_enemy(p_enemy, score, enemy_spawn, events, enemy_color, rng);
                    events.push(GameEvent::Explosion {
                        pos: p_enemy.pos,
                        color: WHITE,
                    });
                }

                map.set(tx, ty, Tile::BreakableWall);
                events.push(GameEvent::Laser {
                    from: origin,
                    to: tile_center,
                    color: WHITE,
                });
                events.push(GameEvent::Shockwave {
                    pos: tile_center,
                    radius: 90.0,
                    color: WHITE,
                });

                if is_circle_colliding(p_active.pos, PLAYER_RADIUS, map, true) {
                    let to_player = p_active.pos - tile_center;
                    let push_dir = if to_player.length_squared() > 0.01 {
                        to_player.normalize()
                    } else {
                        -dir
                    };

                    let safe_distance = TILE_SIZE * 0.5 + PLAYER_RADIUS + 4.0;
                    let candidate_pos = tile_center + push_dir * safe_distance;

                    if !is_circle_colliding(candidate_pos, PLAYER_RADIUS, map, true) {
                        p_active.pos = candidate_pos;
                        events.push(GameEvent::Shockwave {
                            pos: candidate_pos,
                            radius: 60.0,
                            color: my_color,
                        });
                    } else {
                        map.set(tx, ty, Tile::Empty);
                        events.push(GameEvent::Explosion {
                            pos: tile_center,
                            color: WHITE,
                        });
                    }
                }
            }
        }
        Weapon::Stomp => {
            events.push(GameEvent::Shockwave {
                pos: p_active.pos,
                radius: 240.0,
                color: my_color,
            });
            events.push(GameEvent::Shockwave {
                pos: p_active.pos,
                radius: 480.0,
                color: WHITE,
            });
            events.push(GameEvent::Explosion {
                pos: p_active.pos,
                color: GRAY,
            });

            if p_active.pos.distance(p_enemy.pos) < 240.0 {
                kill_enemy(p_enemy, enemy_score, enemy_spawn, events, enemy_color, rng);
            }
        }
    }
}
