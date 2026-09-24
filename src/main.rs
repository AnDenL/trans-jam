use macroquad::prelude::*;

use crate::assets::Assets;
use crate::game::game_loop;
use crate::maps::{TILE_SIZE, Tile, load_default_map};
use crate::simulation::GgrsConfig;
use crate::ui::*;

mod assets;
mod config;
mod game;
mod maps;
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
    let bg = Color::new(0.02, 0.03, 0.05, 1.0);
    let mut state = State::Menu;

    let conf_file = load_file("config.txt");

    let mut username = "username".to_owned();
    let mut port = "7575".to_owned();
    let mut address = "127.0.0.1".to_owned();

    let mut global_map = load_default_map();
    let mut editor_cam = vec2(
        global_map.w as f32 * TILE_SIZE * 0.5,
        global_map.h as f32 * TILE_SIZE * 0.5,
    );
    let mut editor_map_name = "map.txt".to_string();
    let mut editor_selected_tile = Tile::Wall;
    let mut global_assets = Assets::load();

    loop {
        clear_background(bg);

        match state {
            State::Menu => draw_menu(&mut state, &mut username),
            State::Settings => draw_settings(&mut state),
            State::Host => draw_host(&mut state, &mut username, &mut port),
            State::Join => draw_join(&mut state, &mut username, &mut port, &mut address),
            State::Editor => editor_loop(
                &mut state,
                &mut global_map,
                &mut editor_cam,
                &mut editor_map_name,
                &mut editor_selected_tile,
                &global_assets,
            ),
            State::Game(ref mut game) => game_loop(game),
        }

        next_frame().await
    }
}
