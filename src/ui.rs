use std::process::exit;

use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets},
};

use crate::State::{self};

pub fn draw_menu(state: &mut State, username: &mut String) {
    widgets::InputText::new(hash!())
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 290.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui(), username);

    let host = widgets::Button::new("Host")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 230.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    let join = widgets::Button::new("Join")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 170.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    let setting = widgets::Button::new("Settings")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 110.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    let exit_button = widgets::Button::new("Exit")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 50.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    if host {
        *state = State::Host;
    }
    if join {
        *state = State::Join;
    }
    if setting {
        *state = State::Settings;
    }
    if exit_button {
        exit(0);
    }
}

pub fn draw_settings(state: &mut State) {
    let back = widgets::Button::new("Back")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 50.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    if back {
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

    let start = widgets::Button::new("Start")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 110.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    let back = widgets::Button::new("Back")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 50.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    if start {
        *state = State::Menu;
    }

    if back {
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

    let start = widgets::Button::new("Start")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 110.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    let back = widgets::Button::new("Back")
        .position(vec2(
            screen_width() * 0.5 - 100.0,
            screen_width() * 0.5 - 50.0,
        ))
        .size(vec2(200.0, 50.0))
        .ui(&mut *root_ui());

    if start {
        *state = State::Menu;
    }

    if back {
        *state = State::Menu;
    }
}
