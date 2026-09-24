use macroquad::prelude::*;

use crate::PLAYER_RADIUS;
use crate::assets::Assets;
use crate::maps::{Map, TILE_SIZE, Tile};
use crate::player::Player;
use crate::simulation::{GameEvent, Utillity, Weapon};

pub fn draw_drone(pos: Vec2, dir: Vec2, color: Color, parry_timer: u8) {
    if parry_timer > 0 {
        draw_circle_lines(pos.x, pos.y, PLAYER_RADIUS + 10.0, 4.0, GOLD);
        draw_circle(
            pos.x,
            pos.y,
            PLAYER_RADIUS + 8.0,
            Color::new(1.0, 0.84, 0.0, 0.25),
        );
    }

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

pub fn draw_map(map: &Map, assets: &Assets) {
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
                Tile::BreakableWall => {
                    draw_rectangle(
                        px + 2.0,
                        py + 2.0,
                        TILE_SIZE - 4.0,
                        TILE_SIZE - 4.0,
                        Color::new(0.4, 0.32, 0.35, 1.0),
                    );
                    draw_rectangle_lines(
                        px + 2.0,
                        py + 2.0,
                        TILE_SIZE - 4.0,
                        TILE_SIZE - 4.0,
                        2.0,
                        Color::new(0.6, 0.75, 0.9, 1.0),
                    );

                    draw_texture_ex(
                        &assets.crack_tex,
                        px,
                        py,
                        Color::new(0.1, 0.02, 0.05, 1.0),
                        DrawTextureParams {
                            dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
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

pub struct Shockwave {
    pub pos: Vec2,
    pub current_radius: f32,
    pub target_radius: f32,
    pub color: Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Default)]
pub struct VisualEffects {
    pub particles: Vec<Particle>,
    pub lasers: Vec<LaserBeam>,
    pub shockwaves: Vec<Shockwave>,
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

    pub fn spawn_sparks(&mut self, pos: Vec2, color: Color) {
        for _ in 0..16 {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(6.0, 16.0);
            self.particles.push(Particle {
                pos,
                vel: vec2(angle.cos() * speed, angle.sin() * speed),
                color: if rand::gen_range(0, 2) == 0 {
                    WHITE
                } else {
                    color
                },
                size: rand::gen_range(2.0, 5.0),
                lifetime: rand::gen_range(0.15, 0.35),
                max_lifetime: 0.35,
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
                    let duplicate = self.lasers.iter().any(|l| {
                        (l.from - from).length_squared() < 4.0 && (l.to - to).length_squared() < 4.0
                    });
                    if !duplicate {
                        self.lasers.push(LaserBeam {
                            from,
                            to,
                            color,
                            lifetime: 0.12,
                        });
                        self.spawn_sparks(to, color);
                        self.shake_trauma = (self.shake_trauma + 0.25).min(1.0);
                    }
                }
                GameEvent::Blink { from, to, color } => {
                    let duplicate = self.lasers.iter().any(|l| {
                        (l.from - from).length_squared() < 4.0 && (l.to - to).length_squared() < 4.0
                    });
                    if !duplicate {
                        self.lasers.push(LaserBeam {
                            from,
                            to,
                            color: Color::new(color.r, color.g, color.b, 0.4),
                            lifetime: 0.2,
                        });
                        self.spawn_dust(from, color);
                        self.spawn_dust(to, color);
                    }
                }
                GameEvent::Shockwave { pos, radius, color } => {
                    self.shockwaves.push(Shockwave {
                        pos,
                        current_radius: 10.0,
                        target_radius: radius,
                        color,
                        lifetime: 0.25,
                        max_lifetime: 0.25,
                    });
                }
                GameEvent::Explosion { pos, color } => {
                    let duplicate = self
                        .particles
                        .iter()
                        .any(|p| (p.pos - pos).length_squared() < 16.0);
                    if !duplicate {
                        self.spawn_explosion(pos, color);
                        self.shake_trauma = (self.shake_trauma + 0.55).min(1.0);
                    }
                }
                GameEvent::UsbDrop { pos, color } => {
                    self.spawn_dust(pos, color);
                }
                GameEvent::UsbRecall { from, to, color } => {
                    let duplicate = self.lasers.iter().any(|l| {
                        (l.from - from).length_squared() < 4.0 && (l.to - to).length_squared() < 4.0
                    });
                    if !duplicate {
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
                }
                GameEvent::Zipper { from, to, color } => {
                    let duplicate = self.lasers.iter().any(|l| {
                        (l.from - from).length_squared() < 4.0 && (l.to - to).length_squared() < 4.0
                    });
                    if !duplicate {
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
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.pos += p.vel;
            p.vel *= 0.90;
            p.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        for l in &mut self.lasers {
            l.lifetime -= dt;
        }
        self.lasers.retain(|l| l.lifetime > 0.0);

        for s in &mut self.shockwaves {
            s.lifetime -= dt;
            let progress = 1.0 - (s.lifetime / s.max_lifetime).max(0.0);
            s.current_radius = 10.0 + (s.target_radius - 10.0) * progress;
        }
        self.shockwaves.retain(|s| s.lifetime > 0.0);

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

pub fn draw_loadout_hud(player: &Player, my_color: Color) {
    let panel_w = 460.0;
    let panel_h = 100.0;
    let x = 25.0;
    let y = screen_height() - panel_h - 25.0;

    draw_rectangle(x, y, panel_w, panel_h, Color::new(0.02, 0.03, 0.05, 0.85));
    draw_rectangle_lines(
        x,
        y,
        panel_w,
        panel_h,
        2.0,
        Color::new(0.2, 0.25, 0.35, 0.8),
    );

    draw_rectangle(x, y, panel_w, 3.0, my_color);

    let (w_name, w_desc) = match player.weapon {
        Weapon::Railgun => ("RAILGUN", "Pinpoint instant beam"),
        Weapon::Shotgun => ("SHOTGUN", "5-beam close spread"),
        Weapon::Piercing => ("PIERCER", "Passes through walls"),
        Weapon::WhiteOut => ("WHITE OUT", "Spawns 1-hit cover / Crushes"),
        Weapon::NailFile => ("NAIL FILE", "Instant thrust dash"),
        Weapon::Stomp => ("STOMP", "Kills if enemy neaby"),
    };
    draw_text("[LMB] WEAPON", x + 15.0, y + 25.0, 16.0, DARKGRAY);
    draw_text(w_name, x + 15.0, y + 50.0, 22.0, WHITE);
    draw_text(w_desc, x + 15.0, y + 75.0, 14.0, GRAY);

    let (m_name, m_desc) = match player.movement {
        crate::simulation::Movement::Hook => ("HOOK", "Instant pull/blink"),
        crate::simulation::Movement::Zipper => ("ZIPPER", "Phases through walls"),
    };
    let m_x = x + 160.0;
    draw_line(
        m_x - 10.0,
        y + 15.0,
        m_x - 10.0,
        y + panel_h - 15.0,
        1.0,
        Color::new(0.15, 0.2, 0.3, 0.6),
    );
    draw_text("[SPACE] MOVE", m_x, y + 25.0, 16.0, DARKGRAY);
    draw_text(m_name, m_x, y + 50.0, 22.0, WHITE);
    draw_text(m_desc, m_x, y + 75.0, 14.0, GRAY);

    let (u_name, u_desc, u_color) = match player.util {
        Utillity::ZaWarudo => ("ZA WARUDO", "Replay past 60 frames", GOLD),
        Utillity::Usb(None) => ("USB: SET", "Drop recall point", SKYBLUE),
        Utillity::Usb(Some(_)) => ("USB: RECALL", "Teleport to point", GREEN),
        Utillity::Parry => ("PARRY", "Deflect attack", ORANGE),
        Utillity::None => ("SPENT", "Used this life", DARKGRAY),
    };
    let u_x = x + 310.0;
    draw_line(
        u_x - 10.0,
        y + 15.0,
        u_x - 10.0,
        y + panel_h - 15.0,
        1.0,
        Color::new(0.15, 0.2, 0.3, 0.6),
    );
    draw_text("[RMB] UTIL", u_x, y + 25.0, 16.0, DARKGRAY);
    draw_text(u_name, u_x, y + 50.0, 20.0, u_color);
    draw_text(u_desc, u_x, y + 75.0, 14.0, GRAY);
}
