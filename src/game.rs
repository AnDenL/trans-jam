use ggrs::P2PSession;
use macroquad::prelude::*;

use crate::{
    assets::Assets,
    maps::{TILE_SIZE, Tile},
    player::{Input, Player, PlayerInput},
    render::{VisualEffects, draw_drone, draw_loadout_hud, draw_map},
    simulation::{self, GameState, GgrsConfig},
};
use simulation::Utillity;

pub struct ActiveGame {
    pub session: P2PSession<GgrsConfig>,
    pub local_handle: usize,
    pub state: GameState,
    pub effects: VisualEffects,
    pub assets: Assets,
}

impl ActiveGame {
    pub fn new(session: P2PSession<GgrsConfig>, local_handle: usize) -> Self {
        let loaded_map = crate::maps::load_default_map();
        let (p1_spawns, p2_spawns) = loaded_map.get_spawns();
        let p1_spawn = p1_spawns[0];
        let p2_spawn = p2_spawns[0];
        let my_spawn = if local_handle == 0 {
            p1_spawn
        } else {
            p2_spawn
        };

        let mut seed = 13377331u64;
        let p1_weapon = crate::simulation::roll_weapon(&mut seed);
        let p2_weapon = crate::simulation::roll_weapon(&mut seed);
        let p1_move = crate::simulation::roll_movement(&mut seed);
        let p2_move = crate::simulation::roll_movement(&mut seed);
        let p1_util = crate::simulation::roll_util(&mut seed);
        let p2_util = crate::simulation::roll_util(&mut seed);

        Self {
            session,
            local_handle,
            state: GameState {
                p1: Player {
                    pos: p1_spawn,
                    dir: vec2(1.0, 0.0),
                    weapon: p1_weapon,
                    movement: p1_move,
                    util: p1_util,
                    ..Default::default()
                },
                p2: Player {
                    pos: p2_spawn,
                    dir: vec2(-1.0, 0.0),
                    weapon: p2_weapon,
                    movement: p2_move,
                    util: p2_util,
                    ..Default::default()
                },
                score_p1: 0,
                score_p2: 0,
                events: Vec::new(),
                map: loaded_map,
                rng_seed: seed,
                frame_count: 0,
            },
            effects: VisualEffects {
                cam_pos: my_spawn,
                ..Default::default()
            },
            assets: Assets::load(),
        }
    }
}

pub fn game_loop(game: &mut ActiveGame) {
    let dt = get_frame_time();
    game.effects.update(dt);

    let my_pos = match game.local_handle {
        0 => game.state.p1.pos,
        1 => game.state.p2.pos,
        _ => panic!(),
    };

    let target_cam = my_pos + game.effects.shake_offset();
    game.effects.cam_pos = game.effects.cam_pos.lerp(target_cam, 10.0 * dt);

    let view_width = 3600.0;
    let view_height = view_width * (screen_height() / screen_width());

    let camera = Camera2D {
        target: game.effects.cam_pos,
        zoom: vec2(2.0 / view_width, -2.0 / view_height),
        ..Default::default()
    };

    game.session.poll_remote_clients();

    if game.session.current_state() == ggrs::SessionState::Running {
        let mut input = Input::empty();
        if is_key_down(KeyCode::A) {
            input |= Input::LEFT;
        }
        if is_key_down(KeyCode::D) {
            input |= Input::RIGHT;
        }
        if is_key_down(KeyCode::W) {
            input |= Input::UP;
        }
        if is_key_down(KeyCode::S) {
            input |= Input::DOWN;
        }
        if is_key_down(KeyCode::Space) {
            input |= Input::MOVE;
        }
        if is_key_down(KeyCode::LeftShift) {
            input |= Input::DMOVE;
        }
        if is_mouse_button_down(MouseButton::Left) {
            input |= Input::FIRE;
        }
        if is_mouse_button_down(MouseButton::Right) {
            input |= Input::UTIL;
        }

        let mouse_screen = mouse_position();
        let mouse_world = camera.screen_to_world(vec2(mouse_screen.0, mouse_screen.1));
        let diff = mouse_world - my_pos;
        let angle_rad = diff.y.atan2(diff.x);
        let norm = (angle_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
        let aim_angle = (norm * 255.0) as u8;

        game.session
            .add_local_input(
                game.local_handle,
                PlayerInput {
                    buttons: input,
                    aim_angle,
                },
            )
            .unwrap();

        match game.session.advance_frame() {
            Ok(requests) => {
                let mut frame_events = Vec::new();

                for req in requests {
                    match req {
                        ggrs::GgrsRequest::SaveGameState { cell, frame } => {
                            cell.save(frame, Some(game.state.clone()), None);
                        }
                        ggrs::GgrsRequest::LoadGameState { cell, .. } => {
                            game.state = cell.load().unwrap();
                        }
                        ggrs::GgrsRequest::AdvanceFrame { inputs } => {
                            game.state.advance([inputs[0], inputs[1]]);
                            frame_events.append(&mut game.state.events);
                        }
                    }
                }

                game.effects.consume_events(&mut frame_events);
            }
            Err(e) => eprintln!("GGRS error: {:?}", e),
        }
    }

    set_camera(&camera);
    draw_map(&game.state.map, &game.assets);

    if let Utillity::Usb(Some(pos)) = game.state.p1.util {
        draw_rectangle(pos.x - 8.0, pos.y - 14.0, 16.0, 28.0, GOLD);
        draw_rectangle(pos.x - 5.0, pos.y - 20.0, 10.0, 6.0, LIGHTGRAY);
        draw_circle_lines(pos.x, pos.y, 24.0, 1.5, Color::new(1.0, 0.84, 0.0, 0.4));
    }
    if let Utillity::Usb(Some(pos)) = game.state.p2.util {
        draw_rectangle(pos.x - 8.0, pos.y - 14.0, 16.0, 28.0, SKYBLUE);
        draw_rectangle(pos.x - 5.0, pos.y - 20.0, 10.0, 6.0, LIGHTGRAY);
        draw_circle_lines(pos.x, pos.y, 24.0, 1.5, Color::new(0.53, 0.81, 0.92, 0.4));
    }

    for laser in &game.effects.lasers {
        let alpha = (laser.lifetime / 0.12).clamp(0.0, 1.0);
        draw_line(
            laser.from.x,
            laser.from.y,
            laser.to.x,
            laser.to.y,
            8.0 * alpha,
            Color::new(laser.color.r, laser.color.g, laser.color.b, alpha),
        );
        let brightness = laser.color.r * 0.3 + laser.color.g * 0.59 + laser.color.b * 0.11;
        if brightness > 0.15 {
            draw_line(
                laser.from.x,
                laser.from.y,
                laser.to.x,
                laser.to.y,
                2.5 * alpha,
                Color::new(1.0, 1.0, 1.0, alpha * 0.7),
            );
        }
    }

    for p in &game.effects.particles {
        let alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
        draw_rectangle(
            p.pos.x - p.size * 0.5,
            p.pos.y - p.size * 0.5,
            p.size,
            p.size,
            Color::new(p.color.r, p.color.g, p.color.b, alpha),
        );
    }

    for s in &game.effects.shockwaves {
        let alpha = (s.lifetime / s.max_lifetime).clamp(0.0, 1.0);
        draw_circle_lines(
            s.pos.x,
            s.pos.y,
            s.current_radius,
            4.0 * alpha,
            Color::new(s.color.r, s.color.g, s.color.b, alpha),
        );
    }

    let p1_in_smoke = game.state.map.get(
        (game.state.p1.pos.x / TILE_SIZE) as i32,
        (game.state.p1.pos.y / TILE_SIZE) as i32,
    ) == Tile::Smoke;
    let p2_in_smoke = game.state.map.get(
        (game.state.p2.pos.x / TILE_SIZE) as i32,
        (game.state.p2.pos.y / TILE_SIZE) as i32,
    ) == Tile::Smoke;

    let p1_hidden = p1_in_smoke && game.state.p1.reveal_timer == 0;
    let p2_hidden = p2_in_smoke && game.state.p2.reveal_timer == 0;

    if game.local_handle == 0 {
        let alpha = if p1_hidden { 0.4 } else { 1.0 };
        draw_drone(
            game.state.p1.pos,
            game.state.p1.dir,
            Color::new(GOLD.r, GOLD.g, GOLD.b, alpha),
            game.state.p1.parry_timer,
        );
        if !p2_hidden {
            draw_drone(
                game.state.p2.pos,
                game.state.p2.dir,
                SKYBLUE,
                game.state.p2.parry_timer,
            );
        }
    } else {
        let alpha = if p2_hidden { 0.4 } else { 1.0 };
        draw_drone(
            game.state.p2.pos,
            game.state.p2.dir,
            Color::new(SKYBLUE.r, SKYBLUE.g, SKYBLUE.b, alpha),
            game.state.p2.parry_timer,
        );
        if !p1_hidden {
            draw_drone(
                game.state.p1.pos,
                game.state.p1.dir,
                GOLD,
                game.state.p1.parry_timer,
            );
        }
    }
    set_default_camera();

    if game.session.current_state() == ggrs::SessionState::Running {
        let score_text = format!("{}  :  {}", game.state.score_p1, game.state.score_p2);
        let font_size = 46.0;
        let dim = measure_text(&score_text, None, font_size as u16, 1.0);
        let cx = screen_width() * 0.5;

        draw_rectangle(
            cx - dim.width * 0.5 - 25.0,
            15.0,
            dim.width + 50.0,
            50.0,
            Color::new(0.0, 0.0, 0.0, 0.65),
        );
        draw_rectangle_lines(
            cx - dim.width * 0.5 - 25.0,
            15.0,
            dim.width + 50.0,
            50.0,
            2.0,
            DARKGRAY,
        );
        draw_text("P1", cx - dim.width * 0.5 - 80.0, 52.0, 32.0, GOLD);
        draw_text(&score_text, cx - dim.width * 0.5, 52.0, font_size, WHITE);
        draw_text("P2", cx + dim.width * 0.5 + 35.0, 52.0, 32.0, SKYBLUE);

        let my_player = match game.local_handle {
            0 => &game.state.p1,
            1 => &game.state.p2,
            _ => panic!(),
        };

        let my_color = if game.local_handle == 0 {
            GOLD
        } else {
            SKYBLUE
        };
        draw_loadout_hud(my_player, my_color);
    } else {
        draw_text("Waiting for another player...", 50.0, 50.0, 30.0, WHITE);
    }
}
