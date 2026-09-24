use bitflags::bitflags;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{PLAYER_RADIUS, maps::Map, physics::*};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct Input: u8 {
        const UP       = 1 << 0;
        const DOWN     = 1 << 1;
        const LEFT     = 1 << 2;
        const RIGHT    = 1 << 3;
        const PARRY    = 1 << 4;
        const FIRE     = 1 << 5;
        const UTIL  = 1 << 6;
        const MOVE     = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    pub buttons: Input,
    pub aim_angle: u8,
}

#[derive(Default, Clone)]
pub struct Player {
    pub pos: Vec2,
    pub dir: Vec2,
    pub weapon: Weapon,
    pub movement: Movement,
    pub util: Utillity,
}

#[derive(Clone, Debug)]
pub enum GameEvent {
    Laser { from: Vec2, to: Vec2, color: Color },
    Blink { from: Vec2, to: Vec2, color: Color },
    Explosion { pos: Vec2, color: Color },
    UsbDrop { pos: Vec2, color: Color },
    UsbRecall { from: Vec2, to: Vec2, color: Color },
    Zipper { from: Vec2, to: Vec2, color: Color },
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
    Remote,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Movement {
    #[default]
    Hook,
    Zipper,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Utillity {
    #[default]
    Parry,
    Usb(Option<Vec2>),
}

pub fn roll_weapon(rng: &mut u64) -> Weapon {
    match xorshift64(rng) % 4 {
        0 => Weapon::Railgun,
        1 => Weapon::Shotgun,
        2 => Weapon::Remote,
        _ => Weapon::Piercing,
    }
}

pub fn roll_movement(rng: &mut u64) -> Movement {
    match xorshift64(rng) % 3 {
        0 => Movement::Zipper,
        _ => Movement::Hook,
    }
}

pub fn roll_util(rng: &mut u64) -> Utillity {
    match xorshift64(rng) % 3 {
        0 => Utillity::Usb(None),
        _ => Utillity::Parry,
    }
}

fn fire_ray(
    origin: Vec2,
    dir: Vec2,
    max_range: f32,
    p_enemy: &mut Player,
    score: &mut u32,
    enemy_spawn: Vec2,
    events: &mut Vec<GameEvent>,
    my_color: Color,
    enemy_color: Color,
    map: &Map,
    rng: &mut u64,
) {
    let wall_hit = raycast_grid(origin, dir, max_range, map);
    let limit = wall_hit.map(|h| h.distance).unwrap_or(max_range);

    if let Some(hit_dist) = check_circle_hit(origin, dir, limit, p_enemy.pos, PLAYER_RADIUS) {
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
    }
}

fn kill_enemy(
    p_enemy: &mut Player,
    score: &mut u32,
    enemy_spawn: Vec2,
    events: &mut Vec<GameEvent>,
    enemy_color: Color,
    rng: &mut u64,
) {
    *score += 1;
    let death_pos = p_enemy.pos;
    p_enemy.pos = enemy_spawn;
    p_enemy.weapon = roll_weapon(rng);
    p_enemy.movement = roll_movement(rng);
    p_enemy.util = roll_util(rng);

    events.push(GameEvent::Explosion {
        pos: death_pos,
        color: enemy_color,
    });
}

fn apply_movement(pos: &mut Vec2, input: Input, speed: f32, map: &Map) {
    let mut move_x = 0.0;
    if input.contains(Input::LEFT) {
        move_x -= speed;
    }
    if input.contains(Input::RIGHT) {
        move_x += speed;
    }
    if move_x != 0.0 && !is_circle_colliding(vec2(pos.x + move_x, pos.y), PLAYER_RADIUS, map) {
        pos.x += move_x;
    }

    let mut move_y = 0.0;
    if input.contains(Input::UP) {
        move_y += speed;
    }
    if input.contains(Input::DOWN) {
        move_y -= speed;
    }
    if move_y != 0.0 && !is_circle_colliding(vec2(pos.x, pos.y + move_y), PLAYER_RADIUS, map) {
        pos.y += move_y;
    }
}

fn apply_teleport(pos: &mut Vec2, dir: Vec2, max_dist: f32, map: &Map) {
    if let Some(hit) = raycast_grid(*pos, dir, max_dist, map) {
        let target = hit.point + hit.normal * (PLAYER_RADIUS + 2.0);
        if !is_circle_colliding(target, PLAYER_RADIUS, map) {
            *pos = target;
            return;
        }
        let target_fallback = hit.point - dir * (PLAYER_RADIUS + 2.0);
        if !is_circle_colliding(target_fallback, PLAYER_RADIUS, map) {
            *pos = target_fallback;
        }
    }
}

impl GameState {
    pub fn advance(&mut self, inputs: [(PlayerInput, ggrs::InputStatus); 2]) {
        self.events.clear();

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
            &self.map,
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
            &self.map,
            &mut self.rng_seed,
        );
    }
}

fn process_player(
    p_active: &mut Player,
    p_enemy: &mut Player,
    score: &mut u32,
    enemy_score: &mut u32,
    input: PlayerInput,
    events: &mut Vec<GameEvent>,
    my_color: Color,
    enemy_color: Color,
    my_spawn: Vec2,
    enemy_spawn: Vec2,
    map: &Map,
    rng: &mut u64,
) {
    let speed = 9.0;

    let rad =
        (input.aim_angle as f32 / 255.0) * (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
    p_active.dir = vec2(rad.cos(), rad.sin());

    apply_movement(&mut p_active.pos, input.buttons, speed, map);

    if input.buttons.contains(Input::MOVE) {
        let origin = p_active.pos;
        let dir = p_active.dir;

        match p_active.movement {
            Movement::Hook => {
                let old_pos = p_active.pos;
                apply_teleport(&mut p_active.pos, p_active.dir, 1500.0, map);

                events.push(GameEvent::Blink {
                    from: old_pos,
                    to: p_active.pos,
                    color: my_color,
                });
            }
            Movement::Zipper => match crate::physics::calculate_zipper(origin, dir, map) {
                crate::physics::ZipperResult::Teleported(target) => {
                    p_active.pos = target;
                    events.push(GameEvent::Zipper {
                        from: origin,
                        to: target,
                        color: my_color,
                    });
                }
                crate::physics::ZipperResult::FellIntoVoid => {
                    *enemy_score += 1;
                    let death_pos = origin;
                    p_active.pos = my_spawn;
                    events.push(GameEvent::Explosion {
                        pos: death_pos,
                        color: DARKGRAY,
                    });
                }
                crate::physics::ZipperResult::Failed => {}
            },
        }
    }

    if input.buttons.contains(Input::FIRE) {
        let origin = p_active.pos;

        match p_active.weapon {
            Weapon::Railgun => {
                fire_ray(
                    origin,
                    p_active.dir,
                    3000.0,
                    p_enemy,
                    score,
                    enemy_spawn,
                    events,
                    my_color,
                    enemy_color,
                    map,
                    rng,
                );
            }

            Weapon::Shotgun => {
                let base_angle = p_active.dir.y.atan2(p_active.dir.x);
                let spreads = [-0.22, -0.11, 0.0, 0.11, 0.22];

                for offset in spreads {
                    let angle = base_angle + offset;
                    let dir = vec2(angle.cos(), angle.sin());
                    fire_ray(
                        origin,
                        dir,
                        1300.0,
                        p_enemy,
                        score,
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
                let dir = p_active.dir;
                let max_dist = 4000.0;

                if let Some(hit_dist) =
                    check_circle_hit(origin, dir, max_dist, p_enemy.pos, PLAYER_RADIUS)
                {
                    kill_enemy(p_enemy, score, enemy_spawn, events, enemy_color, rng);
                    events.push(GameEvent::Laser {
                        from: origin,
                        to: origin + dir * hit_dist,
                        color: Color::new(1.0, 0.2, 0.8, 1.0),
                    });
                } else {
                    events.push(GameEvent::Laser {
                        from: origin,
                        to: origin + dir * max_dist,
                        color: Color::new(1.0, 0.2, 0.8, 1.0),
                    });
                }
            }
            Weapon::Remote => {
                let dir = p_active.dir;
                let max_dist = 4000.0;
                let wall_hit = raycast_grid(origin, dir, max_dist, map);
                let limit = wall_hit.map(|h| h.distance).unwrap_or(max_dist);

                if let Some(_) = check_circle_hit(origin, dir, limit, p_enemy.pos, PLAYER_RADIUS) {
                    let temp = p_active.pos;
                    p_active.pos = p_enemy.pos;
                    p_enemy.pos = temp;

                    events.push(GameEvent::Blink {
                        from: origin,
                        to: p_enemy.pos,
                        color: my_color,
                    });
                    events.push(GameEvent::Blink {
                        from: p_enemy.pos,
                        to: origin,
                        color: enemy_color,
                    });
                } else {
                    events.push(GameEvent::Laser {
                        from: origin,
                        to: origin + dir * limit,
                        color: Color::new(0.5, 1.0, 0.5, 1.0),
                    });
                }
            }
        }
    }
    if input.buttons.contains(Input::UTIL) {
        match p_active.util {
            Utillity::Parry => todo!(),
            Utillity::Usb(ref mut opt) => match opt {
                None => {
                    *opt = Some(p_active.pos);
                    events.push(GameEvent::UsbDrop {
                        pos: p_active.pos,
                        color: my_color,
                    });
                }
                Some(saved_pos) => {
                    let old_pos = p_active.pos;
                    p_active.pos = *saved_pos;

                    events.push(GameEvent::UsbRecall {
                        from: old_pos,
                        to: p_active.pos,
                        color: my_color,
                    });
                }
            },
        }
    }
}

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    type Input = PlayerInput;
    type InputPredictor = ggrs::PredictRepeatLast;
    type State = GameState;
    type Address = std::net::SocketAddr;
}
