use macroquad::{
    audio::{PlaySoundParams, play_sound, play_sound_once},
    prelude::*,
};

use crate::{
    PLAYER_RADIUS,
    assets::{self, Assets},
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
    pub fn advance(&mut self, inputs: [(PlayerInput, ggrs::InputStatus); 2], assets: &Assets) {
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
            assets,
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
            assets,
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
    Spring,
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
    Dice,
}

pub fn roll_weapon(rng: &mut u64) -> Weapon {
    match xorshift64(rng) % 6 {
        0 => Weapon::Railgun,
        1 => Weapon::Shotgun,
        2 => Weapon::Piercing,
        3 => Weapon::WhiteOut,
        4 => Weapon::Stomp,
        _ => Weapon::Spring,
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
    match xorshift64(rng) % 4 {
        0 => Utillity::Usb(None),
        1 => Utillity::ZaWarudo,
        2 => Utillity::Dice,
        _ => Utillity::Parry,
    }
}

pub fn fire_ray(
    mut origin: Vec2,
    mut dir: Vec2,
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
    assets: &Assets,
) {
    let mut remaining_dist = max_range;

    for _ in 0..3 {
        let wall_hit = raycast_grid(origin, dir, remaining_dist, map);
        let limit = wall_hit.map(|h| h.distance).unwrap_or(remaining_dist);

        if let Some(hit_dist) = check_circle_hit(origin, dir, limit, p_enemy.pos, PLAYER_RADIUS) {
            if p_enemy.parry_timer > 0 {
                kill_enemy(
                    p_active,
                    enemy_score,
                    my_spawn,
                    events,
                    my_color,
                    rng,
                    assets,
                );
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
                return;
            }

            kill_enemy(
                p_enemy,
                score,
                enemy_spawn,
                events,
                enemy_color,
                rng,
                assets,
            );
            events.push(GameEvent::Laser {
                from: origin,
                to: origin + dir * hit_dist,
                color: my_color,
            });
            return;
        }

        let end_point = origin + dir * limit;
        events.push(GameEvent::Laser {
            from: origin,
            to: end_point,
            color: my_color,
        });

        if let Some(hit) = wall_hit {
            match hit.tile_type {
                Tile::Mirror => {
                    events.push(GameEvent::Shockwave {
                        pos: hit.point,
                        radius: 30.0,
                        color: SKYBLUE,
                    });
                    dir = dir - 2.0 * dir.dot(hit.normal) * hit.normal;
                    origin = hit.point + hit.normal * 1.0;
                    remaining_dist -= limit;
                    if remaining_dist <= 20.0 {
                        break;
                    }
                }
                Tile::BreakableWall => {
                    map.set(
                        hit.tile_coord.0 as i32,
                        hit.tile_coord.1 as i32,
                        Tile::Empty,
                    );
                    events.push(GameEvent::Explosion {
                        pos: hit.point,
                        color: WHITE,
                    });
                    break;
                }
                _ => break,
            }
        } else {
            break;
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
    assets: &Assets,
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
                assets,
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
                    800.0,
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
                    assets,
                );
            }
        }

        Weapon::Piercing => {
            let max_dist = 4000.0;

            let volume = rand::gen_range(0.8, 1.0);

            play_sound(
                &assets.snd_explosion,
                PlaySoundParams {
                    looped: false,
                    volume,
                },
            );

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
                        kill_enemy(
                            p_active,
                            enemy_score,
                            my_spawn,
                            events,
                            my_color,
                            rng,
                            assets,
                        );
                        events.push(GameEvent::Shockwave {
                            pos: p_enemy.pos,
                            radius: 160.0,
                            color: GOLD,
                        });
                        return;
                    }

                    kill_enemy(
                        p_enemy,
                        score,
                        enemy_spawn,
                        events,
                        enemy_color,
                        rng,
                        assets,
                    );
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

            let solid_hit = raycast_solid(origin, dir, max_place_dist, map);
            let limit = solid_hit.map(|h| h.distance).unwrap_or(max_place_dist);

            let block_hit = raycast_grid(origin, dir, max_place_dist, map);

            let mut killed = false;
            let mut target_point = match solid_hit {
                Some(h) => h.point - dir * (TILE_SIZE * 0.4),
                None => origin + dir * max_place_dist,
            };

            if let Some(hit_dist) = check_circle_hit(origin, dir, limit, p_enemy.pos, PLAYER_RADIUS)
            {
                if p_enemy.parry_timer > 0 {
                    kill_enemy(
                        p_active,
                        enemy_score,
                        my_spawn,
                        events,
                        my_color,
                        rng,
                        assets,
                    );
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
                    return;
                }

                killed = true;
                let death_spot = p_enemy.pos;
                target_point = death_spot;

                kill_enemy(
                    p_enemy,
                    score,
                    enemy_spawn,
                    events,
                    enemy_color,
                    rng,
                    assets,
                );
                events.push(GameEvent::Explosion {
                    pos: death_spot,
                    color: WHITE,
                });
                events.push(GameEvent::Laser {
                    from: origin,
                    to: origin + dir * hit_dist,
                    color: WHITE,
                });
            } else {
                if let Some(bh) = block_hit {
                    if bh.tile_type == Tile::BreakableWall && bh.distance > (PLAYER_RADIUS + 8.0) {
                        map.set(bh.tile_coord.0 as i32, bh.tile_coord.1 as i32, Tile::Empty);
                        events.push(GameEvent::Explosion {
                            pos: bh.point,
                            color: WHITE,
                        });
                        events.push(GameEvent::Shockwave {
                            pos: bh.point,
                            radius: 80.0,
                            color: LIGHTGRAY,
                        });
                        events.push(GameEvent::Laser {
                            from: origin,
                            to: bh.point,
                            color: WHITE,
                        });
                        return;
                    }
                }
            }

            let tx = (target_point.x / TILE_SIZE).floor() as i32;
            let ty = (target_point.y / TILE_SIZE).floor() as i32;
            let current_tile = map.get(tx, ty);

            let can_place = matches!(
                current_tile,
                Tile::Empty | Tile::Smoke | Tile::SpawnP1 | Tile::SpawnP2
            );

            if can_place {
                let tile_center = vec2(
                    tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                    ty as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                );

                if (p_enemy.pos - tile_center).length() < (PLAYER_RADIUS + TILE_SIZE * 0.45) {
                    if !killed {
                        let death_spot = p_enemy.pos;
                        kill_enemy(
                            p_enemy,
                            score,
                            enemy_spawn,
                            events,
                            enemy_color,
                            rng,
                            assets,
                        );
                        events.push(GameEvent::Explosion {
                            pos: death_spot,
                            color: WHITE,
                        });
                    }
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
                        map.set(tx, ty, current_tile);
                        events.push(GameEvent::Explosion {
                            pos: tile_center,
                            color: WHITE,
                        });
                    }
                }
            }
        }
        Weapon::Stomp => {
            let stomp_radius = 320.0;
            events.push(GameEvent::Shockwave {
                pos: p_active.pos,
                radius: 320.0,
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

            let min_tx = ((p_active.pos.x - stomp_radius) / TILE_SIZE).floor() as i32;
            let max_tx = ((p_active.pos.x + stomp_radius) / TILE_SIZE).floor() as i32;
            let min_ty = ((p_active.pos.y - stomp_radius) / TILE_SIZE).floor() as i32;
            let max_ty = ((p_active.pos.y + stomp_radius) / TILE_SIZE).floor() as i32;

            for ty in min_ty..=max_ty {
                for tx in min_tx..=max_tx {
                    if map.get(tx, ty) == Tile::BreakableWall {
                        let tile_center = vec2(
                            tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                            ty as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                        );

                        if (p_active.pos - tile_center).length() <= stomp_radius + TILE_SIZE * 0.4 {
                            map.set(tx, ty, Tile::Empty);
                            events.push(GameEvent::Explosion {
                                pos: tile_center,
                                color: WHITE,
                            });
                            events.push(GameEvent::Shockwave {
                                pos: tile_center,
                                radius: 75.0,
                                color: LIGHTGRAY,
                            });
                        }
                    }
                }
            }

            if p_active.pos.distance(p_enemy.pos) < stomp_radius {
                if p_enemy.parry_timer > 0 {
                    kill_enemy(
                        p_active,
                        enemy_score,
                        my_spawn,
                        events,
                        my_color,
                        rng,
                        assets,
                    );
                    events.push(GameEvent::Shockwave {
                        pos: p_enemy.pos,
                        radius: 160.0,
                        color: GOLD,
                    });
                    return;
                }

                kill_enemy(
                    p_enemy,
                    score,
                    enemy_spawn,
                    events,
                    enemy_color,
                    rng,
                    assets,
                );
            }
        }
        Weapon::Spring => {
            let max_range = 800.0;
            let push_force = 500.0;
            let wall_hit = raycast_grid(origin, dir, max_range, map);
            if let Some(hit) = wall_hit {
                if hit.tile_type == Tile::BreakableWall {
                    map.set(
                        hit.tile_coord.0 as i32,
                        hit.tile_coord.1 as i32,
                        Tile::Empty,
                    );
                }
            }
            let limit = wall_hit.map(|h| h.distance).unwrap_or(max_range);

            let perp = vec2(-dir.y, dir.x) * 22.0;
            events.push(GameEvent::Laser {
                from: origin - perp,
                to: origin + dir * limit - perp,
                color: Color::new(0.3, 0.95, 0.4, 0.8),
            });
            events.push(GameEvent::Laser {
                from: origin,
                to: origin + dir * limit,
                color: Color::new(0.6, 1.0, 0.6, 1.0),
            });
            events.push(GameEvent::Laser {
                from: origin + perp,
                to: origin + dir * limit + perp,
                color: Color::new(0.3, 0.95, 0.4, 0.8),
            });
            events.push(GameEvent::Shockwave {
                pos: origin + dir * 60.0,
                radius: 110.0,
                color: Color::new(0.3, 0.95, 0.4, 1.0),
            });

            if let Some(_) = check_circle_hit(origin, dir, limit, p_enemy.pos, PLAYER_RADIUS * 2.2)
            {
                if p_enemy.parry_timer > 0 {
                    kill_enemy(
                        p_active,
                        enemy_score,
                        my_spawn,
                        events,
                        my_color,
                        rng,
                        assets,
                    );
                    events.push(GameEvent::Shockwave {
                        pos: p_enemy.pos,
                        radius: 160.0,
                        color: GOLD,
                    });
                    return;
                }

                let start_enemy_pos = p_enemy.pos;
                let enemy_wall_hit =
                    raycast_grid(start_enemy_pos, dir, push_force + PLAYER_RADIUS, map);

                match enemy_wall_hit {
                    Some(hit) if hit.distance <= push_force => {
                        let impact_pos = hit.point - dir * (PLAYER_RADIUS * 0.5);
                        p_enemy.pos = impact_pos;

                        if hit.tile_type == Tile::BreakableWall {
                            map.set(
                                hit.tile_coord.0 as i32,
                                hit.tile_coord.1 as i32,
                                Tile::Empty,
                            );
                        }

                        events.push(GameEvent::Blink {
                            from: start_enemy_pos,
                            to: impact_pos,
                            color: Color::new(0.3, 0.95, 0.4, 0.6),
                        });
                        kill_enemy(
                            p_enemy,
                            score,
                            enemy_spawn,
                            events,
                            enemy_color,
                            rng,
                            assets,
                        );

                        events.push(GameEvent::Explosion {
                            pos: hit.point,
                            color: Color::new(0.3, 0.95, 0.4, 1.0),
                        });
                        events.push(GameEvent::Shockwave {
                            pos: hit.point,
                            radius: 150.0,
                            color: WHITE,
                        });
                    }
                    Some(hit) => {
                        let final_pos = hit.point - dir * (PLAYER_RADIUS + 2.0);
                        events.push(GameEvent::Blink {
                            from: start_enemy_pos,
                            to: final_pos,
                            color: Color::new(0.3, 0.95, 0.4, 0.5),
                        });
                        p_enemy.pos = final_pos;
                        resolve_circle_collision(&mut p_enemy.pos, PLAYER_RADIUS, map, false);
                        events.push(GameEvent::Shockwave {
                            pos: p_enemy.pos,
                            radius: 100.0,
                            color: Color::new(0.3, 0.95, 0.4, 1.0),
                        });
                    }
                    None => {
                        let final_pos = start_enemy_pos + dir * push_force;
                        events.push(GameEvent::Blink {
                            from: start_enemy_pos,
                            to: final_pos,
                            color: Color::new(0.3, 0.95, 0.4, 0.5),
                        });
                        p_enemy.pos = final_pos;
                        resolve_circle_collision(&mut p_enemy.pos, PLAYER_RADIUS, map, false);
                        events.push(GameEvent::Shockwave {
                            pos: p_enemy.pos,
                            radius: 100.0,
                            color: Color::new(0.3, 0.95, 0.4, 1.0),
                        });
                    }
                }
            }

            if let Some(hit) = wall_hit {
                if hit.distance < 360.0 {
                    let recoil_dist = (360.0 - hit.distance).min(280.0) + 100.0;
                    let recoil_dir = -dir;

                    let back_hit =
                        raycast_grid(origin, recoil_dir, recoil_dist + PLAYER_RADIUS, map);
                    let actual_recoil_pos = match back_hit {
                        Some(b_hit) => b_hit.point - recoil_dir * (PLAYER_RADIUS + 2.0),
                        None => origin + recoil_dir * recoil_dist,
                    };

                    events.push(GameEvent::Blink {
                        from: origin,
                        to: actual_recoil_pos,
                        color: WHITE,
                    });
                    p_active.pos = actual_recoil_pos;
                    resolve_circle_collision(&mut p_active.pos, PLAYER_RADIUS, map, false);

                    events.push(GameEvent::Explosion {
                        pos: hit.point,
                        color: WHITE,
                    });
                    events.push(GameEvent::Shockwave {
                        pos: hit.point,
                        radius: 90.0,
                        color: Color::new(0.3, 0.95, 0.4, 1.0),
                    });
                }
            }
        }
    }
}
