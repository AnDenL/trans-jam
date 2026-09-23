use std::process::exit;

use ggrs::{PlayerType, SessionBuilder, UdpNonBlockingSocket};
use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets},
};

use crate::{ActiveGame, FPS, GgrsConfig, State};

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
    if widgets::Button::new("Settings")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 110.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui())
    {
        *state = State::Settings;
    }
    if widgets::Button::new("Exit")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 50.0,
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
