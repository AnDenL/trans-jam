use macroquad::prelude::*;

pub struct Assets {
    pub crack_tex: Texture2D,
}

impl Assets {
    pub fn load() -> Self {
        let crack_bytes = include_bytes!("../assets/crack.png");
        let crack_tex = Texture2D::from_file_with_format(crack_bytes, None);

        crack_tex.set_filter(FilterMode::Linear);

        Self { crack_tex }
    }
}
