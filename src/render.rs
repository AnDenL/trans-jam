use macroquad::prelude::*;

use crate::PLAYER_RADIUS;
use crate::maps::{Map, TILE_SIZE, Tile};
use crate::simulation::GameEvent;

pub fn draw_drone(pos: Vec2, dir: Vec2, color: Color) {
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

pub fn draw_map(map: &Map) {
    for y in 0..map.h {
        for x in 0..map.w {
            let px = x as f32 * TILE_SIZE;
            let py = y as f32 * TILE_SIZE;

            match map.get(x as i32, y as i32) {
                Tile::Wall => {
                    let hash = ((x * 73856093) ^ (y * 19349663)) as u32;

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
                        3.,
                        Color::new(0.25, 0.35, 0.45, 0.1 * (hash % 5) as f32),
                    );
                }
                Tile::Empty => {
                    let hash = ((x * 73856093) ^ (y * 19349663)) as u32;

                    let shade = 0.045 + (hash % 15) as f32 * 0.001;
                    draw_rectangle(
                        px,
                        py,
                        TILE_SIZE,
                        TILE_SIZE,
                        Color::new(shade, shade * 1.2, shade * 1.6, 1.0),
                    );

                    if hash % 5 == 0 {
                        let sub_x = px + ((hash >> 4) % (TILE_SIZE as u32 - 10)) as f32;
                        let sub_y = py + ((hash >> 8) % (TILE_SIZE as u32 - 10)) as f32;
                        draw_rectangle(sub_x, sub_y, 2.0, 2.0, Color::new(0.12, 0.16, 0.22, 0.7));
                    }
                    draw_rectangle_lines(
                        px,
                        py,
                        TILE_SIZE,
                        TILE_SIZE,
                        1.0,
                        Color::new(0.07, 0.09, 0.13, 1.0),
                    );
                }
                Tile::SpawnP1 => {
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
                    draw_circle(
                        px + TILE_SIZE * 0.5,
                        py + TILE_SIZE * 0.5,
                        25.0,
                        Color::new(1.0, 0.84, 0.0, 0.15),
                    );
                    draw_circle_lines(px + TILE_SIZE * 0.5, py + TILE_SIZE * 0.5, 25.0, 2.0, GOLD);
                }
                Tile::SpawnP2 => {
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
                    draw_circle(
                        px + TILE_SIZE * 0.5,
                        py + TILE_SIZE * 0.5,
                        25.0,
                        Color::new(0.53, 0.81, 0.92, 0.15),
                    );
                    draw_circle_lines(
                        px + TILE_SIZE * 0.5,
                        py + TILE_SIZE * 0.5,
                        25.0,
                        2.0,
                        SKYBLUE,
                    );
                }
            }
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
    pub cam_pos: Vec2,
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
                GameEvent::UsbDrop { pos, color } => {
                    self.spawn_dust(pos, color);
                }
                GameEvent::UsbRecall { from, to, color } => {
                    self.lasers.push(LaserBeam {
                        from,
                        to,
                        color: Color::new(color.r, color.g, color.b, 0.7),
                        lifetime: 0.18,
                    });

                    self.spawn_explosion(from, color);
                    self.spawn_dust(to, color);
                    self.shake_trauma = (self.shake_trauma + 0.35).min(1.0);
                }
                GameEvent::Zipper { from, to, color } => {
                    self.lasers.push(LaserBeam {
                        from,
                        to,
                        color: BLACK,
                        lifetime: 0.25,
                    });
                    self.lasers.push(LaserBeam {
                        from,
                        to,
                        color,
                        lifetime: 0.15,
                    });
                    self.spawn_dust(from, color);
                    self.spawn_dust(to, color);
                    self.shake_trauma = (self.shake_trauma + 0.3).min(1.0);
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
