use macroquad::{
    audio::{Sound, load_sound_from_bytes},
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};

pub struct Assets {
    pub crack_tex: Texture2D,
    pub light_tex: Texture2D,
    pub noise_tex: Texture2D,
    pub smoke_mat: Material,

    pub snd_laser: Sound,
    pub snd_explosion: Sound,
    // pub snd_shockwave: Sound,
    // pub snd_parry: Sound,
    // pub snd_blink: Sound,
}

impl Assets {
    pub async fn load() -> Self {
        let crack_bytes = include_bytes!("../assets/crack.png");
        let crack_tex = Texture2D::from_file_with_format(crack_bytes, None);
        crack_tex.set_filter(FilterMode::Linear);

        let light_bytes = include_bytes!("../assets/light.png");
        let light_tex = Texture2D::from_file_with_format(light_bytes, None);
        light_tex.set_filter(FilterMode::Linear);

        let noise_bytes = include_bytes!("../assets/noise.png");
        let noise_tex = Texture2D::from_file_with_format(noise_bytes, None);
        noise_tex.set_filter(FilterMode::Linear);

        let smoke_mat = create_smoke_material();

        let snd_laser = load_sound_from_bytes(include_bytes!("../assets/laser.wav"))
            .await
            .expect("Failed to load laser.wav");
        let snd_explosion = load_sound_from_bytes(include_bytes!("../assets/explosion.wav"))
            .await
            .expect("Failed to load explosion.wav");
        // let snd_shockwave = load_sound_from_bytes(include_bytes!("../assets/shockwave.wav"))
        //     .await
        //     .expect("Failed to load shockwave.wav");
        // let snd_parry = load_sound_from_bytes(include_bytes!("../assets/parry.wav"))
        //     .await
        //     .expect("Failed to load parry.wav");
        // let snd_blink = load_sound_from_bytes(include_bytes!("../assets/blink.wav"))
        //     .await
        //     .expect("Failed to load blink.wav");

        Self {
            crack_tex,
            light_tex,
            noise_tex,
            smoke_mat,

            snd_laser,
            snd_explosion,
        }
    }
}

const SMOKE_VERTEX: &str = r#"
#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying vec2 uv;
varying vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
    color = color0;
}
"#;

const SMOKE_FRAGMENT: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;
varying vec4 color;

uniform sampler2D NoiseTex;
uniform float Time;
uniform vec2 WorldOffset;
uniform float DrawSize;

void main() {
    vec2 world_px = WorldOffset + uv * DrawSize;
    vec2 world_coord = world_px * 0.002;

    vec2 uv1 = world_coord + vec2(Time * 0.015, Time * 0.01);
    float n1 = texture2D(NoiseTex, fract(uv1)).a;

    vec2 uv2 = world_coord * 1.8 - vec2(Time * 0.02, -Time * 0.015);
    float n2 = texture2D(NoiseTex, fract(uv2)).a;

    vec2 uv3 = world_coord * 3.2 + vec2(-Time * 0.03, Time * 0.025);
    float n3 = texture2D(NoiseTex, fract(uv3)).a;

    float smoke_noise = n1 * 0.52 + n2 * 0.33 + n3 * 0.15;

    vec2 local_center_dist = abs(uv - vec2(0.5)) * 2.0;
    float edge_dist = max(local_center_dist.x, local_center_dist.y) + (smoke_noise - 0.5) * 0.45;
    float border_mask = 1.0 - smoothstep(0.45, 0.95, edge_dist);

    float density = smoothstep(0.25, 0.75, smoke_noise) * border_mask;

    vec3 deep_smoke = vec3(0.12, 0.12, 0.14);
    vec3 mid_smoke  = vec3(0.25, 0.28, 0.34);
    vec3 lit_smoke  = vec3(0.50, 0.55, 0.62);

    vec3 final_color = deep_smoke + mix(deep_smoke, mid_smoke, density);
    final_color = mix(final_color, lit_smoke, pow(density, 2.5));

    float alpha = 0.4 * border_mask + density * 0.75;

    gl_FragColor = vec4(final_color, alpha);
}
"#;

pub fn create_smoke_material() -> Material {
    load_material(
        ShaderSource::Glsl {
            vertex: SMOKE_VERTEX,
            fragment: SMOKE_FRAGMENT,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("Time", UniformType::Float1),
                UniformDesc::new("WorldOffset", UniformType::Float2),
                UniformDesc::new("DrawSize", UniformType::Float1),
            ],
            textures: vec!["NoiseTex".to_string()],
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        },
    )
    .expect("Failed to compile smoke shader")
}
