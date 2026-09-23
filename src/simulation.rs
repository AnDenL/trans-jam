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
        const EXPLODE  = 1 << 6;
        const HOOK     = 1 << 7;
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
    pub usb_pos: Option<Vec2>,
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
) {
    let speed = 9.0;

    let rad =
        (input.aim_angle as f32 / 255.0) * (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
    p_active.dir = vec2(rad.cos(), rad.sin());

    apply_movement(&mut p_active.pos, input.buttons, speed, map);

    // if input.buttons.contains(Input::HOOK) {
    //     let old_pos = p_active.pos;
    //     apply_teleport(&mut p_active.pos, p_active.dir, 1500.0, map);

    //     events.push(GameEvent::Blink {
    //         from: old_pos,
    //         to: p_active.pos,
    //         color: my_color,
    //     });
    // }
    if input.buttons.contains(Input::HOOK) {
        let origin = p_active.pos;
        let dir = p_active.dir;

        match crate::physics::calculate_zipper(origin, dir, map) {
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
        }
    }

    if input.buttons.contains(Input::FIRE) {
        let origin = p_active.pos;
        let dir = p_active.dir;
        let wall_hit = raycast_grid(origin, dir, 3000.0, map);
        let max_dist = wall_hit.map(|h| h.distance).unwrap_or(3000.0);

        if let Some(hit_dist) = check_circle_hit(origin, dir, max_dist, p_enemy.pos, PLAYER_RADIUS)
        {
            *score += 1;
            let death_pos = p_enemy.pos;
            p_enemy.pos = enemy_spawn;

            events.push(GameEvent::Explosion {
                pos: death_pos,
                color: enemy_color,
            });
            events.push(GameEvent::Laser {
                from: origin,
                to: origin + dir * hit_dist,
                color: my_color,
            });
        } else {
            events.push(GameEvent::Laser {
                from: origin,
                to: origin + dir * max_dist,
                color: my_color,
            });
        }
    }
    if input.buttons.contains(Input::EXPLODE) {
        match p_active.usb_pos.take() {
            None => {
                p_active.usb_pos = Some(p_active.pos);
                events.push(GameEvent::UsbDrop {
                    pos: p_active.pos,
                    color: my_color,
                });
            }
            Some(saved_pos) => {
                let old_pos = p_active.pos;
                p_active.pos = saved_pos;

                events.push(GameEvent::UsbRecall {
                    from: old_pos,
                    to: saved_pos,
                    color: my_color,
                });
            }
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
