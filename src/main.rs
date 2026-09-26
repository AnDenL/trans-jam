use macroquad::prelude::*;

use crate::assets::Assets;
use crate::game::game_loop;
use crate::maps::{TILE_SIZE, Tile, load_default_map};
use crate::player::Input;
use crate::simulation::GgrsConfig;
use crate::ui::*;

mod assets;
mod config;
mod game;
mod maps;
mod net;
mod physics;
mod player;
mod render;
mod simulation;
mod ui;

pub const FPS: f64 = 60.0;
pub const PLAYER_RADIUS: f32 = 38.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "TransferJam".to_owned(),
        window_width: 1200,
        window_height: 800,
        window_resizable: false,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let bg = Color::new(0.08, 0.10, 0.15, 1.0);
    let mut state = State::Menu;

    //let conf_file = load_file("config.txt");

    let mut map_name = "map.txt".to_owned();
    let mut port = "7575".to_owned();
    let mut address = "127.0.0.1".to_owned();

    let mut global_map = load_default_map();
    let mut editor_cam = vec2(
        global_map.w as f32 * TILE_SIZE * 0.5,
        global_map.h as f32 * TILE_SIZE * 0.5,
    );
    let mut editor_selected_tile = Tile::Wall;
    let global_assets = Assets::load().await;

    loop {
        clear_background(bg);

        if !matches!(state, State::Game(_)) && !matches!(state, State::Editor) {
            draw_menu_background();
        }

        let mut next_state = None;

        match state {
            State::Menu => draw_menu(&mut state),
            State::Settings => draw_settings(&mut state),
            State::Host => draw_host(&mut state, &mut port, &mut map_name),
            State::Join => draw_join(&mut state, &mut port, &mut address),
            State::ConnectingHost(ref mut syncer) => {
                next_state = draw_connecting_host(syncer);
            }
            State::ConnectingJoin(ref mut syncer) => {
                next_state = draw_connecting_join(syncer);
            }
            State::Editor => editor_loop(
                &mut state,
                &mut global_map,
                &mut editor_cam,
                &mut map_name,
                &mut editor_selected_tile,
                &global_assets,
            ),
            State::Game(ref mut game) => game_loop(game, &global_assets),
        }

        if let Some(new_state) = next_state {
            state = new_state;
        }

        next_frame().await
    }
}
