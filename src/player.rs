use bitflags::bitflags;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    PLAYER_RADIUS,
    maps::Map,
    physics::{is_circle_colliding, raycast_grid, resolve_circle_collision},
    simulation::*,
};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct Input: u8 {
        const UP       = 1 << 0;
        const DOWN     = 1 << 1;
        const LEFT     = 1 << 2;
        const RIGHT    = 1 << 3;
        const DMOVE    = 1 << 4;
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

#[derive(Clone)]
pub struct Player {
    pub pos: Vec2,
    pub dir: Vec2,
    pub weapon: Weapon,
    pub movement: Movement,
    pub util: Utillity,
    pub last_buttons: Input,
    pub history: [PlayerInput; 60],
    pub history_idx: usize,
    pub parry_timer: u8,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            dir: Vec2::X,
            weapon: Default::default(),
            movement: Default::default(),
            util: Default::default(),
            last_buttons: Input::empty(),
            history: [PlayerInput::default(); 60],
            history_idx: 0,
            parry_timer: 0,
        }
    }
}

pub fn process_player(
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
    map: &mut Map,
    rng: &mut u64,
) {
    let speed = 12.0;

    if p_active.parry_timer > 0 {
        p_active.parry_timer -= 1;
    }

    let rad =
        (input.aim_angle as f32 / 255.0) * (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
    p_active.dir = vec2(rad.cos(), rad.sin());

    let move_dir = get_move_dir(input.buttons, speed);
    let can_phase_through = p_active.weapon == Weapon::WhiteOut;
    apply_movement(&mut p_active.pos, move_dir, map, can_phase_through);

    let just_pressed_move =
        input.buttons.contains(Input::MOVE) && !p_active.last_buttons.contains(Input::MOVE);
    let just_pressed_dmove =
        input.buttons.contains(Input::DMOVE) && !p_active.last_buttons.contains(Input::DMOVE);

    let just_pressed_fire =
        input.buttons.contains(Input::FIRE) && !p_active.last_buttons.contains(Input::FIRE);
    let just_pressed_util =
        input.buttons.contains(Input::UTIL) && !p_active.last_buttons.contains(Input::UTIL);

    p_active.history[p_active.history_idx % 60] = input;
    p_active.history_idx += 1;

    if just_pressed_move || just_pressed_dmove {
        let origin = p_active.pos; //

        let dir = if just_pressed_dmove && move_dir.length_squared() > 0.001 {
            move_dir.normalize()
        } else {
            p_active.dir
        };

        match p_active.movement {
            Movement::Hook => {
                let old_pos = p_active.pos;
                apply_teleport(&mut p_active.pos, dir, 1500.0, map);

                events.push(GameEvent::Blink {
                    from: old_pos,
                    to: p_active.pos,
                    color: my_color,
                });
                events.push(GameEvent::Explosion {
                    pos: p_active.pos,
                    color: my_color,
                });
                events.push(GameEvent::Shockwave {
                    pos: p_active.pos,
                    radius: 150.0,
                    color: WHITE,
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

    if just_pressed_fire {
        execute_weapon(
            p_active.weapon,
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
    if just_pressed_util {
        match p_active.util {
            Utillity::Parry => {
                p_active.parry_timer = 2;
                events.push(GameEvent::Shockwave {
                    pos: p_active.pos,
                    radius: 120.0,
                    color: GOLD,
                });
            }
            Utillity::Usb(None) => {
                events.push(GameEvent::UsbDrop {
                    pos: p_active.pos,
                    color: my_color,
                });
                p_active.util = Utillity::Usb(Some(p_active.pos));
            }
            Utillity::Usb(Some(saved_pos)) => {
                let old_pos = p_active.pos;
                p_active.pos = saved_pos;

                events.push(GameEvent::UsbRecall {
                    from: old_pos,
                    to: p_active.pos,
                    color: my_color,
                });
                p_active.util = Utillity::Usb(None);
            }
            Utillity::ZaWarudo => {
                let old_pos = p_active.pos;

                for i in 0..60 {
                    let past_input = p_active.history[(p_active.history_idx + i) % 60];

                    apply_movement(
                        &mut p_active.pos,
                        get_move_dir(past_input.buttons, speed),
                        map,
                        false,
                    );

                    if past_input.buttons.contains(Input::FIRE) {
                        let rad = (past_input.aim_angle as f32 / 255.0)
                            * (2.0 * std::f32::consts::PI)
                            - std::f32::consts::PI;
                        let past_dir = vec2(rad.cos(), rad.sin());

                        execute_weapon(
                            p_active.weapon,
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

                events.push(GameEvent::Blink {
                    from: old_pos,
                    to: p_active.pos,
                    color: WHITE,
                });
                events.push(GameEvent::Explosion {
                    pos: p_active.pos,
                    color: my_color,
                });

                p_active.util = Utillity::None;
            }
            Utillity::None => {}
        }
    }
    p_active.last_buttons = input.buttons;
}

pub fn kill_enemy(
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
    p_enemy.last_buttons = Input::empty();

    events.push(GameEvent::Explosion {
        pos: death_pos,
        color: enemy_color,
    });
    events.push(GameEvent::Shockwave {
        pos: death_pos,
        radius: 130.0,
        color: enemy_color,
    });
}

pub fn get_move_dir(input: Input, speed: f32) -> Vec2 {
    let mut dir = Vec2::ZERO;
    if input.contains(Input::LEFT) {
        dir.x -= 1.0;
    }
    if input.contains(Input::RIGHT) {
        dir.x += 1.0;
    }
    if input.contains(Input::UP) {
        dir.y += 1.0;
    }
    if input.contains(Input::DOWN) {
        dir.y -= 1.0;
    }

    if dir.length_squared() > 0.001 {
        dir.normalize() * speed
    } else {
        Vec2::ZERO
    }
}

pub fn apply_movement(pos: &mut Vec2, dir: Vec2, map: &Map, ignore_breakable: bool) {
    if dir.length_squared() < 0.0001 {
        return;
    }
    *pos += dir;
    resolve_circle_collision(pos, PLAYER_RADIUS, map, ignore_breakable);
}

pub fn apply_teleport(pos: &mut Vec2, dir: Vec2, max_dist: f32, map: &Map) -> bool {
    if dir.length_squared() < 0.0001 {
        return false;
    }
    let dir = dir.normalize();

    if let Some(hit) = raycast_grid(*pos, dir, max_dist, map) {
        if hit.distance <= PLAYER_RADIUS + 4.0 {
            return false;
        }

        let mut target = hit.point + hit.normal * (PLAYER_RADIUS + 1.0);
        resolve_circle_collision(&mut target, PLAYER_RADIUS, map, false);

        if (*pos - target).length() > 8.0 && !is_circle_colliding(target, PLAYER_RADIUS, map, false)
        {
            *pos = target;
            return true;
        }

        let mut target_fallback = hit.point - dir * (PLAYER_RADIUS + 2.0);
        resolve_circle_collision(&mut target_fallback, PLAYER_RADIUS, map, false);

        if (*pos - target_fallback).length() > 8.0
            && !is_circle_colliding(target_fallback, PLAYER_RADIUS, map, false)
        {
            *pos = target_fallback;
            return true;
        }
    }
    false
}
