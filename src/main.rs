use bitflags::bitflags;
use ggrs::P2PSession;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ui::{draw_host, draw_join, draw_menu, draw_settings};

mod ui;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct Input: u8 {
        const UP     = 1 << 0;
        const DOWN   = 1 << 1;
        const LEFT   = 1 << 2;
        const RIGHT  = 1 << 3;
        const PARRY  = 1 << 4;
    }
}

pub struct GameState {
    // your game state here
    session: P2PSession<GgrsConfig>,
}

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    type Input = Input; // transmitted over the network each frame
    type InputPredictor = ggrs::PredictRepeatLast; // how to predict missing remote inputs
    type State = GameState; // saved/loaded during rollbacks
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

enum State {
    Menu,
    Settings,
    Host,
    Join,
    Game(Box<GameState>),
}

#[macroquad::main(window_conf)]
async fn main() {
    let bg = Color::new(0.0, 0.05, 0.1, 1.0);
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
            State::Game => todo!(),
        }

        next_frame().await
    }
}
