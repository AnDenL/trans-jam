use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets},
};
use std::process::exit;

use crate::{
    assets::Assets,
    game::ActiveGame,
    maps::{Map, TILE_SIZE, Tile},
    net::{HostPoll, HostSyncer, JoinPoll, JoinSyncer},
    render::draw_map,
};

pub enum State {
    Menu,
    Settings,
    Host,
    Join,
    ConnectingHost(Box<HostSyncer>),
    ConnectingJoin(Box<JoinSyncer>),
    Editor,
    Game(Box<ActiveGame>),
}

pub fn draw_menu_background() {
    let t = get_time() as f32;

    let cx = screen_width() * 0.5;
    let cy = screen_height() * 0.5;
    let pulse = (t * 2.0).sin() * 15.0;

    draw_circle_lines(
        cx,
        cy,
        260.0 + pulse,
        10.0,
        Color::new(0.15, 0.25, 0.35, 0.25),
    );
    draw_circle_lines(cx, cy, 320.0 - pulse, 15.0, Color::new(0.3, 0.5, 0.7, 0.15));
}

pub fn draw_menu(state: &mut State) {
    let cx = screen_width() * 0.5 - 100.0;
    let cy = screen_height() * 0.5 - 160.0;

    if widgets::Button::new("Host")
        .position(vec2(cx, cy))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Host;
    }
    if widgets::Button::new("Join")
        .position(vec2(cx, cy + 65.0))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Join;
    }
    if widgets::Button::new("Editor")
        .position(vec2(cx, cy + 130.0))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Editor;
    }
    if widgets::Button::new("Settings")
        .position(vec2(cx, cy + 195.0))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Settings;
    }
    if widgets::Button::new("Exit")
        .position(vec2(cx, cy + 260.0))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        exit(0);
    }
}

pub fn draw_settings(state: &mut State) {
    let cx = screen_width() * 0.5 - 100.0;
    let cy = screen_height() * 0.5;

    if widgets::Button::new("Back")
        .position(vec2(cx, cy))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Menu;
    }
}

pub fn draw_host(state: &mut State, port: &mut String, map_name: &mut String) {
    let cx = screen_width() * 0.5 - 110.0;
    let cy = screen_height() * 0.5 - 130.0;

    draw_text("HOST GAME", cx + 35.0, cy - 20.0, 28.0, WHITE);

    draw_text("Port:", cx, cy + 15.0, 18.0, LIGHTGRAY);
    widgets::InputText::new(hash!())
        .position(vec2(cx, cy + 25.0))
        .size(vec2(220.0, 40.0))
        .ui(&mut *root_ui(), port);

    draw_text("Map file:", cx, cy + 85.0, 18.0, LIGHTGRAY);
    widgets::InputText::new(hash!())
        .position(vec2(cx, cy + 55.0))
        .size(vec2(220.0, 40.0))
        .ui(&mut *root_ui(), map_name);

    if widgets::Button::new("Start")
        .position(vec2(cx, cy + 160.0))
        .size(vec2(220.0, 45.0))
        .ui(&mut *root_ui())
    {
        match HostSyncer::new(port, map_name) {
            Ok(syncer) => *state = State::ConnectingHost(Box::new(syncer)),
            Err(e) => eprintln!("Host Init Error: {}", e),
        }
    }

    if widgets::Button::new("Back")
        .position(vec2(cx, cy + 215.0))
        .size(vec2(220.0, 40.0))
        .ui(&mut *root_ui())
    {
        *state = State::Menu;
    }
}

pub fn draw_join(state: &mut State, port: &mut String, address: &mut String) {
    let cx = screen_width() * 0.5 - 110.0;
    let cy = screen_height() * 0.5 - 130.0;

    draw_text("JOIN GAME", cx + 40.0, cy - 20.0, 28.0, WHITE);

    draw_text("Host IP:", cx, cy + 15.0, 18.0, LIGHTGRAY);
    widgets::InputText::new(hash!())
        .position(vec2(cx, cy + 25.0))
        .size(vec2(220.0, 40.0))
        .ui(&mut *root_ui(), address);

    draw_text("Host Port:", cx, cy + 85.0, 18.0, LIGHTGRAY);
    widgets::InputText::new(hash!())
        .position(vec2(cx, cy + 55.0))
        .size(vec2(220.0, 40.0))
        .ui(&mut *root_ui(), port);

    if widgets::Button::new("Connect")
        .position(vec2(cx, cy + 160.0))
        .size(vec2(220.0, 45.0))
        .ui(&mut *root_ui())
    {
        let my_local_port = "7576";
        match JoinSyncer::new(my_local_port, address, port) {
            Ok(syncer) => *state = State::ConnectingJoin(Box::new(syncer)),
            Err(e) => eprintln!("Join Init Error: {}", e),
        }
    }

    if widgets::Button::new("Back")
        .position(vec2(cx, cy + 215.0))
        .size(vec2(220.0, 40.0))
        .ui(&mut *root_ui())
    {
        *state = State::Menu;
    }
}

pub fn draw_connecting_host(syncer: &mut HostSyncer) -> Option<State> {
    let dt = get_frame_time();
    let poll = syncer.update(dt);

    let cx = screen_width() * 0.5;
    let cy = screen_height() * 0.5;

    let dots = match ((get_time() * 3.0) as usize) % 4 {
        1 => ".",
        2 => "..",
        3 => "...",
        _ => "",
    };

    draw_rectangle(
        cx - 180.0,
        cy - 100.0,
        360.0,
        220.0,
        Color::new(0.04, 0.06, 0.1, 0.95),
    );
    draw_rectangle_lines(cx - 180.0, cy - 100.0, 360.0, 220.0, 2.0, SKYBLUE);

    match poll {
        HostPoll::WaitingForClient => {
            let text = format!("Waiting for player{}", dots);
            draw_text(&text, cx - 120.0, cy - 40.0, 24.0, WHITE);
            draw_text(
                &format!("Listening on port: {}", syncer.port),
                cx - 110.0,
                cy - 10.0,
                18.0,
                GRAY,
            );
        }
        HostPoll::SendingMap { total } => {
            draw_text("Client connected!", cx - 90.0, cy - 45.0, 22.0, GREEN);
            draw_text(
                &format!("Syncing map ({} chunks){}", total, dots),
                cx - 110.0,
                cy - 15.0,
                18.0,
                SKYBLUE,
            );
        }
        HostPoll::Ready(game) => {
            return Some(State::Game(game));
        }
        HostPoll::Failed(err) => {
            draw_text("Error!", cx - 35.0, cy - 45.0, 22.0, RED);
            draw_text(&err, cx - 140.0, cy - 15.0, 16.0, WHITE);
        }
    }

    if widgets::Button::new("Cancel")
        .position(vec2(cx - 75.0, cy + 35.0))
        .size(vec2(150.0, 40.0))
        .ui(&mut *root_ui())
    {
        return Some(State::Host);
    }

    None
}

pub fn draw_connecting_join(syncer: &mut JoinSyncer) -> Option<State> {
    let dt = get_frame_time();
    let poll = syncer.update(dt);

    let cx = screen_width() * 0.5;
    let cy = screen_height() * 0.5;

    let dots = match ((get_time() * 3.0) as usize) % 4 {
        1 => ".",
        2 => "..",
        3 => "...",
        _ => "",
    };

    draw_rectangle(
        cx - 180.0,
        cy - 100.0,
        360.0,
        220.0,
        Color::new(0.04, 0.06, 0.1, 0.95),
    );
    draw_rectangle_lines(cx - 180.0, cy - 100.0, 360.0, 220.0, 2.0, GOLD);

    match poll {
        JoinPoll::Connecting => {
            let text = format!("Connecting to host{}", dots);
            draw_text(&text, cx - 115.0, cy - 40.0, 24.0, WHITE);
            draw_text(
                &format!("Target: {}", syncer.host_addr),
                cx - 110.0,
                cy - 10.0,
                18.0,
                GRAY,
            );
        }
        JoinPoll::ReceivingMap { received, total } => {
            draw_text("Downloading map...", cx - 95.0, cy - 45.0, 22.0, SKYBLUE);
            draw_text(
                &format!("Chunks: {} / {}", received, total),
                cx - 55.0,
                cy - 15.0,
                18.0,
                WHITE,
            );
        }
        JoinPoll::Ready(game) => {
            return Some(State::Game(game));
        }
        JoinPoll::Failed(err) => {
            draw_text("Failed!", cx - 35.0, cy - 45.0, 22.0, RED);
            draw_text(&err, cx - 140.0, cy - 15.0, 16.0, WHITE);
        }
    }

    if widgets::Button::new("Cancel")
        .position(vec2(cx - 75.0, cy + 35.0))
        .size(vec2(150.0, 40.0))
        .ui(&mut *root_ui())
    {
        return Some(State::Join);
    }

    None
}

pub fn editor_loop(
    state: &mut State,
    map: &mut Map,
    cam_pos: &mut Vec2,
    map_name: &mut String,
    selected_tile: &mut Tile,
    assets: &Assets,
) {
    let bg = Color::new(0.1, 0.1, 0.2, 1.0);
    clear_background(bg);

    let cam_speed = 15.0;
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        cam_pos.x += cam_speed;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        cam_pos.x -= cam_speed;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        cam_pos.y -= cam_speed;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        cam_pos.y += cam_speed;
    }

    let view_width = 3000.0;
    let view_height = view_width * (screen_height() / screen_width());
    let camera = Camera2D {
        target: *cam_pos,
        zoom: vec2(2.0 / view_width, -2.0 / view_height),
        ..Default::default()
    };

    set_camera(&camera);
    draw_map(map, assets);

    let mouse_screen = mouse_position();
    let mouse_world = camera.screen_to_world(vec2(mouse_screen.0, mouse_screen.1));
    let tx = (mouse_world.x / TILE_SIZE).floor() as i32;
    let ty = (mouse_world.y / TILE_SIZE).floor() as i32;

    if tx >= 0 && tx < map.w as i32 && ty >= 0 && ty < map.h as i32 {
        draw_rectangle_lines(
            tx as f32 * TILE_SIZE,
            ty as f32 * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
            3.0,
            RED,
        );
    }
    set_default_camera();

    widgets::Window::new(hash!(), vec2(10., 10.), vec2(280., 440.))
        .label("Map Tools")
        .ui(&mut *root_ui(), |ui| {
            ui.input_text(hash!(), "File", map_name);

            if ui.button(None, "Save Map") {
                map.save(map_name);
            }
            if ui.button(None, "Load Map") {
                *map = Map::load(map_name);
            }

            ui.separator();
            ui.label(None, &format!("Size: {} x {}", map.w, map.h));
            if ui.button(None, "Width +1") {
                map.resize(map.w + 1, map.h);
            }
            if ui.button(None, "Width -1") {
                map.resize(map.w.saturating_sub(1).max(5), map.h);
            }
            if ui.button(None, "Height +1") {
                map.resize(map.w, map.h + 1);
            }
            if ui.button(None, "Height -1") {
                map.resize(map.w, map.h.saturating_sub(1).max(5));
            }

            ui.separator();
            ui.label(None, "Current Brush:");
            if ui.button(
                None,
                if *selected_tile == Tile::Wall {
                    "[x] Wall"
                } else {
                    "Wall"
                },
            ) {
                *selected_tile = Tile::Wall;
            }
            if ui.button(
                None,
                if *selected_tile == Tile::BreakableWall {
                    "[x] Breakable wall"
                } else {
                    "Breakable wall"
                },
            ) {
                *selected_tile = Tile::BreakableWall;
            }
            if ui.button(
                None,
                if *selected_tile == Tile::Mirror {
                    "[x] Mirror"
                } else {
                    "Mirror"
                },
            ) {
                *selected_tile = Tile::Mirror;
            }
            if ui.button(
                None,
                if *selected_tile == Tile::Glass {
                    "[x] Glass"
                } else {
                    "Glass"
                },
            ) {
                *selected_tile = Tile::Glass;
            }
            if ui.button(
                None,
                if *selected_tile == Tile::Smoke {
                    "[x] Smoke"
                } else {
                    "Smoke"
                },
            ) {
                *selected_tile = Tile::Smoke;
            }
            if ui.button(
                None,
                if *selected_tile == Tile::SpawnP1 {
                    "[x] Spawn P1"
                } else {
                    "Spawn P1"
                },
            ) {
                *selected_tile = Tile::SpawnP1;
            }
            if ui.button(
                None,
                if *selected_tile == Tile::SpawnP2 {
                    "[x] Spawn P2"
                } else {
                    "Spawn P2"
                },
            ) {
                *selected_tile = Tile::SpawnP2;
            }

            ui.separator();
            if ui.button(None, "Back to Menu") {
                *state = State::Menu;
            }
        });

    let ui_rect = Rect::new(10.0, 10.0, 280.0, 480.0);
    let mouse_over_ui = ui_rect.contains(vec2(mouse_screen.0, mouse_screen.1));

    if !root_ui().is_mouse_captured() && !mouse_over_ui {
        if tx >= 0 && tx < map.w as i32 && ty >= 0 && ty < map.h as i32 {
            if is_mouse_button_down(MouseButton::Left) {
                map.set(tx, ty, *selected_tile);
            } else if is_mouse_button_down(MouseButton::Right) {
                map.set(tx, ty, Tile::Empty);
            }
        }
    }
}
