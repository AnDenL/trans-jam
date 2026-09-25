use ggrs::{PlayerType, SessionBuilder, UdpNonBlockingSocket};
use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets},
};
use std::process::exit;

use crate::{
    FPS, GgrsConfig,
    maps::{Map, TILE_SIZE, Tile},
    render::draw_map,
};
use crate::{assets::Assets, game::ActiveGame};

pub enum State {
    Menu,
    Settings,
    Host,
    Join,
    Editor,
    Game(Box<ActiveGame>),
}

pub fn draw_menu(state: &mut State, username: &mut String) {
    widgets::InputText::new(hash!())
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 290.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui(), username);

    if widgets::Button::new("Host")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 230.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Host;
    }
    if widgets::Button::new("Join")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 170.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Join;
    }
    if widgets::Button::new("Editor")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 110.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Editor;
    }
    if widgets::Button::new("Settings")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 50.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Settings;
    }
    if widgets::Button::new("Exit")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 + 10.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        exit(0);
    }
}

pub fn draw_settings(state: &mut State) {
    if widgets::Button::new("Back")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 50.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Menu;
    }
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

    widgets::Window::new(hash!(), vec2(10., 10.), vec2(280., 400.))
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

    if !root_ui().is_mouse_captured() {
        if tx >= 0 && tx < map.w as i32 && ty >= 0 && ty < map.h as i32 {
            if is_mouse_button_down(MouseButton::Left) {
                map.set(tx, ty, *selected_tile);
            } else if is_mouse_button_down(MouseButton::Right) {
                map.set(tx, ty, Tile::Empty);
            }
        }
    }
}

pub fn draw_join(
    state: &mut State,
    username: &mut String,
    port: &mut String,
    address: &mut String,
) {
    widgets::InputText::new(hash!())
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 290.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui(), username);
    widgets::InputText::new(hash!())
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 280.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui(), port);
    widgets::InputText::new(hash!())
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 270.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui(), address);

    if widgets::Button::new("Start")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 80.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        let my_local_port = "7576";
        let game = start_join(my_local_port, address, port);
        *state = State::Game(Box::new(game));
    }
    if widgets::Button::new("Back")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 20.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Menu;
    }
}

pub fn draw_host(state: &mut State, username: &mut String, port: &mut String) {
    widgets::InputText::new(hash!())
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 290.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui(), username);
    widgets::InputText::new(hash!())
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 280.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui(), port);

    if widgets::Button::new("Start")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 140.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        let game = start_host(&port);
        *state = State::Game(Box::new(game));
    }
    if widgets::Button::new("Back")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 80.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Menu;
    }
}

fn start_host(port_str: &str) -> ActiveGame {
    let port: u16 = port_str.parse().unwrap();
    let socket = UdpNonBlockingSocket::bind_to_port(port).expect("Port is occupied!");
    let session = SessionBuilder::<GgrsConfig>::new()
        .with_num_players(2)
        .unwrap()
        .with_fps(FPS as usize)
        .unwrap()
        .with_input_delay(1)
        .add_player(PlayerType::Local, 0)
        .unwrap()
        .add_player(PlayerType::Remote("127.0.0.1:7576".parse().unwrap()), 1)
        .unwrap()
        .start_p2p_session(socket)
        .unwrap();
    ActiveGame::new(session, 0)
}

fn start_join(local_port_str: &str, host_addr_str: &str, host_port_str: &str) -> ActiveGame {
    let local_port: u16 = local_port_str.parse().expect("Incorrect local port");
    let host_port: u16 = host_port_str.parse().expect("Incorrect hosts port");
    let host_socket_addr = format!("{}:{}", host_addr_str, host_port)
        .parse()
        .expect("Incorrect IP format");
    let socket = UdpNonBlockingSocket::bind_to_port(local_port).expect("Port is occupied!");
    let session = SessionBuilder::<GgrsConfig>::new()
        .with_num_players(2)
        .unwrap()
        .with_fps(FPS as usize)
        .unwrap()
        .with_input_delay(1)
        .add_player(PlayerType::Remote(host_socket_addr), 0)
        .unwrap()
        .add_player(PlayerType::Local, 1)
        .unwrap()
        .start_p2p_session(socket)
        .unwrap();
    ActiveGame::new(session, 1)
}
