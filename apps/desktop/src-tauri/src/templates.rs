use rustlab::doc_hint::{cavegame_voxel_starter_template, mini_paint_starter_template};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Template {
    #[default]
    Basic,
    Paint,
    Cavegame,
}

impl Template {
    pub fn build(self) -> (&'static str, &'static str, Option<&'static str>) {
        match self {
            Self::Basic => ("", "fn main() {\n    println!(\"Olá, Rust!\");\n}\n", None),
            Self::Paint => (
                "minifb = \"=0.27.0\"\n",
                include_str!("../templates/paint.rs"),
                Some(mini_paint_starter_template()),
            ),
            Self::Cavegame => (
                "bevy = { version = \"=0.20.1\", default-features = false, features = [\"3d\"] }\n",
                include_str!("../templates/cavegame.rs"),
                Some(cavegame_voxel_starter_template()),
            ),
        }
    }
}
