use bitflags::bitflags;
use ggrs::P2PSession;
use macroquad::prelude::*;
use macroquad::rand;
use serde::{Deserialize, Serialize};

use crate::{
    physics::raycast_grid,
    ui::{draw_host, draw_join, draw_menu, draw_settings},
};

mod maps;
mod physics;
mod ui;

pub const FPS: f64 = 60.0;

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
}

#[derive(Clone, Debug)]
pub enum GameEvent {
    Laser { from: Vec2, to: Vec2, color: Color },
    Blink { from: Vec2, to: Vec2, color: Color },
    Explosion { pos: Vec2, color: Color },
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub p1: Player,
    pub p2: Player,
    pub score_p1: u32,
    pub score_p2: u32,
    pub events: Vec<GameEvent>,
    pub map: Map,
}

fn check_circle_hit(
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

fn apply_movement(pos: &mut Vec2, input: Input, speed: f32) {
    let mut move_x = 0.0;
    if input.contains(Input::LEFT) {
        move_x -= speed;
    }
    if input.contains(Input::RIGHT) {
        move_x += speed;
    }
    if move_x != 0.0 && !is_circle_colliding(vec2(pos.x + move_x, pos.y), PLAYER_RADIUS) {
        pos.x += move_x;
    }

    let mut move_y = 0.0;
    if input.contains(Input::UP) {
        move_y += speed;
    }
    if input.contains(Input::DOWN) {
        move_y -= speed;
    }
    if move_y != 0.0 && !is_circle_colliding(vec2(pos.x, pos.y + move_y), PLAYER_RADIUS) {
        pos.y += move_y;
    }
}

fn apply_teleport(pos: &mut Vec2, dir: Vec2, max_dist: f32) {
    if let Some(hit) = raycast_grid(*pos, dir, max_dist, &MAP_TILES) {
        let target = hit.point + hit.normal * (PLAYER_RADIUS + 2.0);
        if !is_circle_colliding(target, PLAYER_RADIUS) {
            *pos = target;
            return;
        }
        let target_fallback = hit.point - dir * (PLAYER_RADIUS + 2.0);
        if !is_circle_colliding(target_fallback, PLAYER_RADIUS) {
            *pos = target_fallback;
        }
    }
}

impl GameState {
    pub fn advance(&mut self, inputs: [(PlayerInput, ggrs::InputStatus); 2]) {
        self.events.clear();

        process_player(
            &mut self.p1,
            &mut self.p2,
            &mut self.score_p1,
            inputs[0].0,
            &mut self.events,
            GOLD,
            SKYBLUE,
            P2_SPAWN,
        );

        process_player(
            &mut self.p2,
            &mut self.p1,
            &mut self.score_p2,
            inputs[1].0,
            &mut self.events,
            SKYBLUE,
            GOLD,
            P1_SPAWN,
        );
    }
}

fn process_player(
    p_active: &mut Player,
    p_enemy: &mut Player,
    score: &mut u32,
    input: PlayerInput,
    events: &mut Vec<GameEvent>,
    my_color: Color,
    enemy_color: Color,
    enemy_spawn: Vec2,
) {
    let speed = 9.0;

    let rad =
        (input.aim_angle as f32 / 255.0) * (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
    p_active.dir = vec2(rad.cos(), rad.sin());

    apply_movement(&mut p_active.pos, input.buttons, speed);

    if input.buttons.contains(Input::HOOK) {
        let old_pos = p_active.pos;
        apply_teleport(&mut p_active.pos, p_active.dir, 3000.0);

        events.push(GameEvent::Blink {
            from: old_pos,
            to: p_active.pos,
            color: my_color,
        });
    }

    if input.buttons.contains(Input::FIRE) {
        let origin = p_active.pos;
        let dir = p_active.dir;
        let wall_hit = raycast_grid(origin, dir, 3000.0, &MAP_TILES);
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
}

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct LaserBeam {
    pub from: Vec2,
    pub to: Vec2,
    pub color: Color,
    pub lifetime: f32,
}

#[derive(Default)]
pub struct VisualEffects {
    pub particles: Vec<Particle>,
    pub lasers: Vec<LaserBeam>,
    pub shake_trauma: f32,
}

impl VisualEffects {
    pub fn spawn_explosion(&mut self, pos: Vec2, color: Color) {
        for _ in 0..45 {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(4.0, 18.0);
            let size = rand::gen_range(4.0, 8.0);
            let lifetime = rand::gen_range(0.3, 0.7);
            self.particles.push(Particle {
                pos,
                vel: vec2(angle.cos() * speed, angle.sin() * speed),
                color: if rand::gen_range(0, 3) == 0 {
                    WHITE
                } else {
                    color
                },
                size,
                lifetime,
                max_lifetime: lifetime,
            });
        }
    }

    pub fn spawn_dust(&mut self, pos: Vec2, color: Color) {
        for _ in 0..8 {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(1.0, 5.0);
            self.particles.push(Particle {
                pos,
                vel: vec2(angle.cos() * speed, angle.sin() * speed),
                color,
                size: rand::gen_range(2.0, 5.0),
                lifetime: rand::gen_range(0.1, 0.3),
                max_lifetime: 0.3,
            });
        }
    }

    pub fn consume_events(&mut self, events: &mut Vec<GameEvent>) {
        for ev in events.drain(..) {
            match ev {
                GameEvent::Laser { from, to, color } => {
                    self.lasers.push(LaserBeam {
                        from,
                        to,
                        color,
                        lifetime: 0.12,
                    });
                    self.shake_trauma = (self.shake_trauma + 0.25).min(1.0);
                }
                GameEvent::Blink { from, to, color } => {
                    self.lasers.push(LaserBeam {
                        from,
                        to,
                        color: Color::new(color.r, color.g, color.b, 0.4),
                        lifetime: 0.2,
                    });
                    self.spawn_dust(from, color);
                    self.spawn_dust(to, color);
                }
                GameEvent::Explosion { pos, color } => {
                    self.spawn_explosion(pos, color);
                    self.shake_trauma = (self.shake_trauma + 0.55).min(1.0);
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.pos += p.vel;
            p.vel *= 0.91;
            p.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        for l in &mut self.lasers {
            l.lifetime -= dt;
        }
        self.lasers.retain(|l| l.lifetime > 0.0);

        if self.shake_trauma > 0.0 {
            self.shake_trauma = (self.shake_trauma - dt * 2.0).max(0.0);
        }
    }

    pub fn shake_offset(&self) -> Vec2 {
        let shake = self.shake_trauma * self.shake_trauma * 22.0;
        if shake > 0.001 {
            vec2(
                rand::gen_range(-shake, shake),
                rand::gen_range(-shake, shake),
            )
        } else {
            Vec2::ZERO
        }
    }
}

pub struct ActiveGame {
    pub session: P2PSession<GgrsConfig>,
    pub local_handle: usize,
    pub state: GameState,
    pub effects: VisualEffects,
}

impl ActiveGame {
    pub fn new(session: P2PSession<GgrsConfig>, local_handle: usize) -> Self {
        Self {
            session,
            local_handle,
            state: GameState {
                p1: Player {
                    pos: P1_SPAWN,
                    dir: vec2(1.0, 0.0),
                },
                p2: Player {
                    pos: P2_SPAWN,
                    dir: vec2(-1.0, 0.0),
                },
                score_p1: 0,
                score_p2: 0,
                events: Vec::new(),
            },
            effects: VisualEffects::default(),
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

fn window_conf() -> Conf {
    Conf {
        window_title: "TransferJam".to_owned(),
        window_width: 1200,
        window_height: 800,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

pub enum State {
    Menu,
    Settings,
    Host,
    Join,
    Game(Box<ActiveGame>),
}

#[macroquad::main(window_conf)]
async fn main() {
    let bg = Color::new(0.02, 0.03, 0.05, 1.0);
    let mut state = State::Menu;

    let mut username = "username".to_owned();
    let mut port = "7575".to_owned();
    let mut address = "127.0.0.1".to_owned();

    loop {
        clear_background(bg);

        match state {
            State::Menu => draw_menu(&mut state, &mut username),
            State::Settings => draw_settings(&mut state),
            State::Host => draw_host(&mut state, &mut username, &mut port),
            State::Join => draw_join(&mut state, &mut username, &mut port, &mut address),
            State::Game(ref mut game) => game_loop(game),
        }

        next_frame().await
    }
}

fn game_loop(game: &mut ActiveGame) {
    let dt = get_frame_time();
    game.effects.update(dt);

    let world_w = MAP_W as f32 * TILE_SIZE;
    let world_h = MAP_H as f32 * TILE_SIZE;

    let camera = Camera2D {
        target: vec2(world_w * 0.5, world_h * 0.5) + game.effects.shake_offset(),
        zoom: vec2(2.0 / world_w, -2.0 / world_h),
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

        if is_key_pressed(KeyCode::Space) {
            input |= Input::HOOK;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            input |= Input::FIRE;
        }

        let my_player = match game.local_handle {
            0 => &game.state.p1,
            1 => &game.state.p2,
            _ => panic!(),
        };

        let mouse_screen = mouse_position();
        let mouse_world = camera.screen_to_world(vec2(mouse_screen.0, mouse_screen.1));

        let diff = mouse_world - my_player.pos;
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
                            game.effects.consume_events(&mut game.state.events);
                        }
                    }
                }
            }
            Err(e) => eprintln!("GGRS error: {:?}", e),
        }
    }

    set_camera(&camera);
    draw_map();

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
        draw_line(
            laser.from.x,
            laser.from.y,
            laser.to.x,
            laser.to.y,
            3.0 * alpha,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
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

    draw_drone(game.state.p1.pos, game.state.p1.dir, GOLD);
    draw_drone(game.state.p2.pos, game.state.p2.dir, SKYBLUE);

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
    } else {
        draw_text("Waiting for another player...", 50.0, 50.0, 30.0, WHITE);
    }
}

fn draw_drone(pos: Vec2, dir: Vec2, color: Color) {
    draw_circle_lines(pos.x, pos.y, PLAYER_RADIUS + 2.0, 2.5, WHITE);
    draw_circle(pos.x, pos.y, PLAYER_RADIUS, color);

    let eye_pos = pos + dir * 18.0;
    draw_circle(
        eye_pos.x,
        eye_pos.y,
        11.0,
        Color::new(0.08, 0.08, 0.12, 1.0),
    );
    draw_circle(eye_pos.x + dir.x * 2.0, eye_pos.y + dir.y * 2.0, 4.5, RED);
}

#[inline(always)]
pub fn is_wall(x: i32, y: i32) -> bool {
    if x < 0 || x >= MAP_W as i32 || y < 0 || y >= MAP_H as i32 {
        return true;
    }
    MAP_TILES[y as usize * MAP_W + x as usize]
}

pub fn is_circle_colliding(pos: Vec2, radius: f32) -> bool {
    let min_tx = ((pos.x - radius) / TILE_SIZE).floor() as i32;
    let max_tx = ((pos.x + radius) / TILE_SIZE).floor() as i32;
    let min_ty = ((pos.y - radius) / TILE_SIZE).floor() as i32;
    let max_ty = ((pos.y + radius) / TILE_SIZE).floor() as i32;

    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            if is_wall(tx, ty) {
                let tile_min_x = tx as f32 * TILE_SIZE;
                let tile_max_x = (tx + 1) as f32 * TILE_SIZE;
                let tile_min_y = ty as f32 * TILE_SIZE;
                let tile_max_y = (ty + 1) as f32 * TILE_SIZE;

                let closest_x = pos.x.clamp(tile_min_x, tile_max_x);
                let closest_y = pos.y.clamp(tile_min_y, tile_max_y);

                let dx = pos.x - closest_x;
                let dy = pos.y - closest_y;
                if dx * dx + dy * dy < radius * radius {
                    return true;
                }
            }
        }
    }
    false
}

pub fn draw_map() {
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let px = x as f32 * TILE_SIZE;
            let py = y as f32 * TILE_SIZE;

            if MAP_TILES[y * MAP_W + x] {
                draw_rectangle(
                    px,
                    py,
                    TILE_SIZE,
                    TILE_SIZE,
                    Color::new(0.12, 0.15, 0.2, 1.0),
                );
                draw_rectangle_lines(
                    px,
                    py,
                    TILE_SIZE,
                    TILE_SIZE,
                    1.5,
                    Color::new(0.25, 0.35, 0.45, 1.0),
                );
            } else {
                draw_rectangle(
                    px,
                    py,
                    TILE_SIZE,
                    TILE_SIZE,
                    Color::new(0.04, 0.05, 0.08, 1.0),
                );
                draw_rectangle_lines(
                    px,
                    py,
                    TILE_SIZE,
                    TILE_SIZE,
                    1.0,
                    Color::new(0.07, 0.09, 0.13, 1.0),
                );
            }
        }
    }
}
