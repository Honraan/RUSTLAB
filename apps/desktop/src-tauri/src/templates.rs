/// Templates curados para criação de projetos independentes no Modo Criar.
pub struct ProjectTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub cargo_toml_extra: &'static str,
    pub main_rs: &'static str,
}

pub fn list_templates() -> Vec<ProjectTemplate> {
    vec![
        ProjectTemplate {
            id: "hello",
            name: "Aplicativo Console (Hello Rust)",
            description: "Projeto Cargo mínimo com testes unitários prontos.",
            cargo_toml_extra: "",
            main_rs: r#"fn saudar(nome: &str) -> String {
    format!("Olá, {nome}! Bem-vindo ao RustLab.")
}

fn main() {
    println!("{}", saudar("estudante"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testa_saudacao() {
        assert!(saudar("Rust").contains("Rust"));
    }
}
"#,
        },
        ProjectTemplate {
            id: "pong",
            name: "Pong no Terminal (D10)",
            description: "Núcleo PongState com colisão, placar e loop interativo ANSI.",
            cargo_toml_extra: "",
            main_rs: rustlab::doc_hint::pong_terminal_starter_template(),
        },
        ProjectTemplate {
            id: "snake",
            name: "Snake no Terminal (D11)",
            description: "Núcleo SnakeGame com seed determinística, colisões e testes.",
            cargo_toml_extra: "",
            main_rs: rustlab::doc_hint::snake_terminal_starter_template(),
        },
        ProjectTemplate {
            id: "editor",
            name: "Mini Editor de Texto (D12)",
            description: "Editor multi-linha com undo/redo e suíte de testes.",
            cargo_toml_extra: "",
            main_rs: rustlab::doc_hint::mini_editor_starter_template(),
        },
        ProjectTemplate {
            id: "paint_gui",
            name: "Mini Paint GUI (eframe 0.31)",
            description: "Aplicação gráfica desktop com Canvas2D preservado no resize e exportação PPM.",
            cargo_toml_extra: "\n[dependencies]\neframe = \"0.31\"\n",
            main_rs: PAINT_GUI_TEMPLATE,
        },
        ProjectTemplate {
            id: "cavegame_bevy",
            name: "CaveGame Voxel 3D (Bevy 0.20)",
            description: "Mundo voxel 24x24 chunks, primeira pessoa, pulo e blocos com Bevy 0.20.",
            cargo_toml_extra: "\n[dependencies]\nbevy = \"0.20\"\n\n[profile.dev]\nopt-level = 1\n\n[profile.dev.package.\"*\"]\nopt-level = 3\n",
            main_rs: CAVEGAME_BEVY_TEMPLATE,
        },
    ]
}

pub fn find_template(id: &str) -> Option<ProjectTemplate> {
    list_templates().into_iter().find(|t| t.id == id)
}

const PAINT_GUI_TEMPLATE: &str = r#"use eframe::egui;

/// Canvas lógico 2D independente do tamanho da janela (PAINT-01..12).
pub struct Canvas2D {
    pub largura: usize,
    pub altura: usize,
    pub pixels: Vec<[u8; 3]>,
    pub undo_stack: Vec<Vec<[u8; 3]>>,
}

impl Canvas2D {
    pub fn novo(largura: usize, altura: usize) -> Self {
        let largura = largura.max(1);
        let altura = altura.max(1);
        Self {
            largura,
            altura,
            pixels: vec![[255, 255, 255]; largura * altura],
            undo_stack: Vec::new(),
        }
    }

    pub fn salvar_undo(&mut self) {
        self.undo_stack.push(self.pixels.clone());
    }

    pub fn desfazer(&mut self) -> bool {
        let Some(prev) = self.undo_stack.pop() else {
            return false;
        };
        self.pixels = prev;
        true
    }

    pub fn limpar(&mut self) {
        self.salvar_undo();
        self.pixels.fill([255, 255, 255]);
    }

    pub fn pintar_pixel(&mut self, x: usize, y: usize, cor: [u8; 3]) {
        if x < self.largura && y < self.altura {
            self.pixels[y * self.largura + x] = cor;
        }
    }

    pub fn tracar_linha(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, cor: [u8; 3]) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            if x0 >= 0 && y0 >= 0 {
                self.pintar_pixel(x0 as usize, y0 as usize, cor);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn mapear_retangulo(&self, rect: egui::Rect, pos: egui::Pos2) -> Option<(i32, i32)> {
        if !rect.contains(pos) || rect.width() <= 0.0 || rect.height() <= 0.0 {
            return None;
        }
        let nx = ((pos.x - rect.left()) / rect.width()) * self.largura as f32;
        let ny = ((pos.y - rect.top()) / rect.height()) * self.altura as f32;
        Some((
            (nx.floor() as i32).clamp(0, self.largura as i32 - 1),
            (ny.floor() as i32).clamp(0, self.altura as i32 - 1),
        ))
    }

    pub fn exportar_ppm(&self) -> String {
        let mut out = format!("P3\n{} {}\n255\n", self.largura, self.altura);
        for [r, g, b] in &self.pixels {
            out.push_str(&format!("{r} {g} {b}\n"));
        }
        out
    }
}

struct MiniPaintApp {
    canvas: Canvas2D,
    cor: [u8; 3],
    ultimo_ponto: Option<(i32, i32)>,
    status: String,
}

impl Default for MiniPaintApp {
    fn default() -> Self {
        Self {
            canvas: Canvas2D::novo(96, 64),
            cor: [220, 72, 32],
            ultimo_ponto: None,
            status: "Arraste no quadro para desenhar. Redimensione a janela sem perder seu desenho!".into(),
        }
    }
}

impl eframe::App for MiniPaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("barra").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Mini Paint (RustLab)");
                if ui.button("Laranja").clicked() {
                    self.cor = [220, 72, 32];
                }
                if ui.button("Azul").clicked() {
                    self.cor = [40, 120, 220];
                }
                if ui.button("Preto").clicked() {
                    self.cor = [24, 24, 28];
                }
                if ui.button("Desfazer").clicked() {
                    self.canvas.desfazer();
                }
                if ui.button("Limpar").clicked() {
                    self.canvas.limpar();
                }
                if ui.button("Exportar PPM").clicked() {
                    let ppm = self.canvas.exportar_ppm();
                    if std::fs::write("desenho.ppm", ppm).is_ok() {
                        self.status = "Exportado para desenho.ppm".into();
                    }
                }
            });
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(available, egui::Sense::drag());
            let painter = ui.painter_at(rect);
            let cell_w = rect.width() / self.canvas.largura as f32;
            let cell_h = rect.height() / self.canvas.altura as f32;
            for y in 0..self.canvas.altura {
                for x in 0..self.canvas.largura {
                    let [r, g, b] = self.canvas.pixels[y * self.canvas.largura + x];
                    let p_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.left() + x as f32 * cell_w, rect.top() + y as f32 * cell_h),
                        egui::vec2(cell_w + 0.5, cell_h + 0.5),
                    );
                    painter.rect_filled(p_rect, 0.0, egui::Color32::from_rgb(r, g, b));
                }
            }
            if response.drag_started() {
                self.canvas.salvar_undo();
                self.ultimo_ponto = response
                    .interact_pointer_pos()
                    .and_then(|p| self.canvas.mapear_retangulo(rect, p));
            }
            if response.dragged()
                && let Some(pos) = response.interact_pointer_pos()
                && let Some(atual) = self.canvas.mapear_retangulo(rect, pos)
            {
                let prev = self.ultimo_ponto.unwrap_or(atual);
                self.canvas.tracar_linha(prev.0, prev.1, atual.0, atual.1, self.cor);
                self.ultimo_ponto = Some(atual);
            }
            if response.drag_stopped() {
                self.ultimo_ponto = None;
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions::default();
    eframe::run_native("Mini Paint — RustLab", opts, Box::new(|_| Ok(Box::<MiniPaintApp>::default())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_preserva_desenho_no_resize_e_exporta() {
        let mut c = Canvas2D::novo(32, 24);
        c.salvar_undo();
        c.tracar_linha(0, 0, 10, 10, [10, 20, 30]);
        assert_eq!(c.pixels[5 * 32 + 5], [10, 20, 30]);
        assert_eq!(c.mapear_retangulo(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(320.0, 240.0)), egui::pos2(160.0, 120.0)), Some((16, 12)));
        assert_eq!(c.mapear_retangulo(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(640.0, 480.0)), egui::pos2(320.0, 240.0)), Some((16, 12)));
        assert!(c.exportar_ppm().starts_with("P3\n32 24\n255\n"));
        assert!(c.desfazer());
        assert_eq!(c.pixels[5 * 32 + 5], [255, 255, 255]);
    }
}
"#;

const CAVEGAME_BEVY_TEMPLATE: &str = r#"use bevy::prelude::*;
use std::collections::BTreeMap;

pub const WORLD_CHUNKS_X: i32 = 24;
pub const WORLD_CHUNKS_Z: i32 = 24;
pub const CHUNK_SIZE: i32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Voxel {
    Ar,
    Grama,
    Terra,
    Pedra,
}

#[derive(Resource)]
pub struct VoxelWorld {
    pub seed: u64,
    pub blocos: BTreeMap<(i32, i32, i32), Voxel>,
}

impl VoxelWorld {
    pub fn novo_com_seed(seed: u64) -> Self {
        Self {
            seed,
            blocos: BTreeMap::new(),
        }
    }

    pub fn altura_terreno(seed: u64, x: i32, z: i32) -> i32 {
        let h = seed
            .wrapping_add((x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
            .wrapping_add((z as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
        2 + ((h >> 12) % 6) as i32
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, v: Voxel) {
        if v == Voxel::Ar {
            self.blocos.remove(&(x, y, z));
        } else {
            self.blocos.insert((x, y, z), v);
        }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> Voxel {
        self.blocos.get(&(x, y, z)).copied().unwrap_or(Voxel::Ar)
    }

    pub fn gerar_area_inicial(&mut self, raio: i32) {
        for x in -raio..=raio {
            for z in -raio..=raio {
                let h = Self::altura_terreno(self.seed, x, z);
                for y in 0..=h {
                    let v = if y == h { Voxel::Grama } else if y + 2 >= h { Voxel::Terra } else { Voxel::Pedra };
                    self.set(x, y, z, v);
                }
            }
        }
    }
}

#[derive(Component)]
struct Jogador {
    vel_y: f32,
    no_chao: bool,
}

fn setup_mundo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut world = VoxelWorld::novo_com_seed(42);
    world.gerar_area_inicial(10);

    let cubo = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mat_grama = materials.add(Color::srgb(0.28, 0.68, 0.32));
    let mat_terra = materials.add(Color::srgb(0.52, 0.36, 0.24));
    let mat_pedra = materials.add(Color::srgb(0.55, 0.56, 0.6));

    for (&(x, y, z), &v) in &world.blocos {
        let mat = match v {
            Voxel::Grama => mat_grama.clone(),
            Voxel::Terra => mat_terra.clone(),
            Voxel::Pedra => mat_pedra.clone(),
            Voxel::Ar => continue,
        };
        commands.spawn((
            Mesh3d(cubo.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(x as f32, y as f32, z as f32),
        ));
    }

    commands.insert_resource(world);
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(12.0, 24.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 14.0).looking_at(Vec3::new(0.0, 4.0, 0.0), Vec3::Y),
        Jogador {
            vel_y: 0.0,
            no_chao: false,
        },
    ));
}

fn mover_jogador(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut Jogador)>,
) {
    let dt = time.delta_secs();
    for (mut tf, mut player) in &mut q {
        let mut dir = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) { dir.z -= 1.0; }
        if keys.pressed(KeyCode::KeyS) { dir.z += 1.0; }
        if keys.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { dir.x += 1.0; }
        if dir.length_squared() > 0.0 {
            tf.translation += dir.normalize() * 8.0 * dt;
        }
        if keys.just_pressed(KeyCode::Space) && player.no_chao {
            player.vel_y = 7.5;
            player.no_chao = false;
        }
        player.vel_y -= 18.0 * dt;
        tf.translation.y += player.vel_y * dt;
        if tf.translation.y < 6.0 {
            tf.translation.y = 6.0;
            player.vel_y = 0.0;
            player.no_chao = true;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_mundo)
        .add_systems(Update, mover_jogador)
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terreno_deterministico_e_colocacao_blocos() {
        let mut w = VoxelWorld::novo_com_seed(42);
        w.gerar_area_inicial(4);
        assert!(w.get(0, 0, 0) != Voxel::Ar);
        w.set(0, 20, 0, Voxel::Pedra);
        assert_eq!(w.get(0, 20, 0), Voxel::Pedra);
        w.set(0, 20, 0, Voxel::Ar);
        assert_eq!(w.get(0, 20, 0), Voxel::Ar);
    }
}
"#;
