//! Glossário offline bilíngue (pt-BR / en-US) para símbolos Rust, bibliotecas extras
//! visuais/de jogos e variáveis/funções definidas no código do aluno.

use crate::i18n::bi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocHint {
    pub summary: &'static str,
    pub url: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDoc {
    pub title: String,
    pub summary: String,
    pub url_pt: Option<String>,
    pub url_en: String,
}

impl ResolvedDoc {
    pub fn best_url(&self) -> &str {
        self.url_pt.as_deref().unwrap_or(&self.url_en)
    }

    pub fn best_url_for(&self, english: bool) -> &str {
        if english {
            &self.url_en
        } else {
            self.best_url()
        }
    }
}

/// Biblioteca visual, de áudio, efeitos ou mini engine pronta para adicionar
/// ao projeto do estudante em um clique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtraLibrary {
    pub name: &'static str,
    pub version: &'static str,
    pub category: &'static str,
    pub summary: &'static str,
    pub docs_pt: &'static str,
    pub key_items: &'static str,
    pub use_line: &'static str,
    pub example: &'static str,
    pub doc_url: &'static str,
}

pub fn extra_libraries(english: bool) -> &'static [ExtraLibrary] {
    if english {
        &EXTRA_LIBRARIES_EN
    } else {
        &EXTRA_LIBRARIES
    }
}

pub const EXTRA_LIBRARIES: [ExtraLibrary; 7] = [
    ExtraLibrary {
        name: "macroquad",
        version: "0.4",
        category: "Mini Engine de Jogos 2D",
        summary: "Engine leve estilo game loop (Pong, plataforma, arcade) com janela, formas 2D, texto e teclado/mouse sem setup complicado.",
        docs_pt: "Abre uma janela gráfica própria (Linux e Windows). Use #[macroquad::main(\"Jogo\")] async fn main(), desenhe com draw_rectangle/draw_circle/draw_text, leia teclas com is_key_down e finalize o frame com next_frame().await.",
        key_items: "clear_background, draw_circle, draw_rectangle, draw_line, draw_text, is_key_down, mouse_position, next_frame",
        use_line: "use macroquad::prelude::*;",
        example: "#[macroquad::main(\"Meu Jogo\")]\nasync fn main() {\n    loop {\n        clear_background(BLACK);\n        draw_circle(screen_width() / 2.0, screen_height() / 2.0, 20.0, GOLD);\n        next_frame().await;\n    }\n}",
        doc_url: "https://docs.rs/macroquad/latest/macroquad/",
    },
    ExtraLibrary {
        name: "bevy",
        version: "0.20",
        category: "Engine de Jogos 2D/3D (ECS)",
        summary: "Engine profissional orientada a dados (Entities, Components, Systems) para jogos 2D e 3D em Rust.",
        docs_pt: "Usa arquitetura ECS: componentes são structs com #[derive(Component)], sistemas são funções comuns (fn mover(mut query: Query<&mut Transform>, tempo: Res<Time>)) registradas em App::new().add_plugins(DefaultPlugins). Pode rodar em janela própria ou em outro terminal (F4).",
        key_items: "App, DefaultPlugins, Startup, Update, Commands, Query, Res, ResMut, Component, Resource, Transform, Camera2d, Sprite, ButtonInput, KeyCode, Time",
        use_line: "use bevy::prelude::*;",
        example: "#[derive(Component)]\nstruct Jogador;\n\nfn main() {\n    App::new()\n        .add_plugins(DefaultPlugins)\n        .add_systems(Startup, setup)\n        .add_systems(Update, mover)\n        .run();\n}",
        doc_url: "https://docs.rs/bevy/latest/bevy/",
    },
    ExtraLibrary {
        name: "minifb",
        version: "0.27",
        category: "Janela Gráfica & Pixels 2D",
        summary: "Janela gráfica simples por framebuffer de pixels (Vec<u32>), ótima para aprender renderização e simulações visuais.",
        docs_pt: "Cria uma janela 2D com Window::new e atualiza um vetor de pixels RGB (0xRRGGBB) em cada frame usando update_with_buffer.",
        key_items: "Window, WindowOptions, Key, update_with_buffer, is_open",
        use_line: "use minifb::{Key, Window, WindowOptions};",
        example: "let mut janela = Window::new(\"Demo\", 320, 200, WindowOptions::default()).unwrap();\nlet buffer = vec![0x0022_4466u32; 320 * 200];\nwhile janela.is_open() && !janela.is_key_down(Key::Escape) {\n    janela.update_with_buffer(&buffer, 320, 200).unwrap();\n}",
        doc_url: "https://docs.rs/minifb/latest/minifb/",
    },
    ExtraLibrary {
        name: "eframe",
        version: "0.31",
        category: "Interface Gráfica (GUI egui)",
        summary: "Framework imediato para criar janelas nativas com botões, textos, sliders e gráficos em desktop.",
        docs_pt: "Permite construir aplicativos visuais com botões, campos de texto e painéis imediatos usando egui::CentralPanel::default().show(...).",
        key_items: "egui, run_native, CentralPanel, SidePanel, Ui, heading, button, label, Slider",
        use_line: "use eframe::egui;",
        example: "egui::CentralPanel::default().show(ctx, |ui| {\n    ui.heading(\"Meu App Visual em Rust!\");\n    if ui.button(\"Clique aqui\").clicked() { /* ... */ }\n});",
        doc_url: "https://docs.rs/eframe/latest/eframe/",
    },
    ExtraLibrary {
        name: "tachyonfx",
        version: "0.11",
        category: "Efeitos Visuais & Animações",
        summary: "Biblioteca composável de efeitos visuais e animações (dissolve, fade, sweep) para interfaces no terminal.",
        docs_pt: "Cria animações fluidas por interpolação com fx::dissolve, fx::fade_from_fg, fx::sweep_in, EffectTimer e Interpolation.",
        key_items: "fx, Effect, EffectTimer, Interpolation, dissolve, fade_from_fg, sweep_in",
        use_line: "use tachyonfx::{Effect, EffectTimer, Interpolation, fx};",
        example: "let efeito = fx::fade_from_fg(ratatui::style::Color::Yellow, (600, Interpolation::SineOut));",
        doc_url: "https://docs.rs/tachyonfx/latest/tachyonfx/",
    },
    ExtraLibrary {
        name: "rand",
        version: "0.9",
        category: "Sorteios & Aleatoriedade",
        summary: "Gera números aleatórios, posições de inimigos, powerups e sorteios em jogos e simulações.",
        docs_pt: "Use rand::rng().random_range(min..max) para sortear números ou random_bool(prob) para eventos com probabilidade.",
        key_items: "Rng, rng, random_range, random_bool",
        use_line: "use rand::Rng;",
        example: "let mut rng = rand::rng();\nlet sorteio = rng.random_range(1..=10);",
        doc_url: "https://docs.rs/rand/latest/rand/",
    },
    ExtraLibrary {
        name: "rodio",
        version: "0.20",
        category: "Áudio & Efeitos Sonoros",
        summary: "Reproduz tons sintetizados, músicas e efeitos sonoros em jogos e aplicativos Rust.",
        docs_pt: "Abre a saída de áudio padrão com OutputStream::try_default() e controla sons com Sink (ex.: SineWave para bipes de colisão).",
        key_items: "OutputStream, Sink, SineWave, Source",
        use_line: "use rodio::{OutputStream, Sink, source::SineWave};",
        example: "if let Ok((_stream, handle)) = OutputStream::try_default() {\n    if let Ok(sink) = Sink::try_new(&handle) { /* tocar som */ }\n}",
        doc_url: "https://docs.rs/rodio/latest/rodio/",
    },
];

pub const EXTRA_LIBRARIES_EN: [ExtraLibrary; 7] = [
    ExtraLibrary {
        name: "macroquad",
        version: "0.4",
        category: "2D Mini Game Engine",
        summary: "Lightweight game-loop engine (Pong, platformers, arcade) with windowing, 2D shapes, text, and keyboard/mouse input without heavy setup.",
        docs_pt: "Opens a dedicated graphical window (Linux and Windows). Use #[macroquad::main(\"Game\")] async fn main(), draw with draw_rectangle/draw_circle/draw_text, read keys with is_key_down, and finish frames with next_frame().await.",
        key_items: "clear_background, draw_circle, draw_rectangle, draw_line, draw_text, is_key_down, mouse_position, next_frame",
        use_line: "use macroquad::prelude::*;",
        example: "#[macroquad::main(\"My Game\")]\nasync fn main() {\n    loop {\n        clear_background(BLACK);\n        draw_circle(screen_width() / 2.0, screen_height() / 2.0, 20.0, GOLD);\n        next_frame().await;\n    }\n}",
        doc_url: "https://docs.rs/macroquad/latest/macroquad/",
    },
    ExtraLibrary {
        name: "bevy",
        version: "0.20",
        category: "2D/3D Game Engine (ECS)",
        summary: "Data-driven game engine (Entities, Components, Systems) for 2D and 3D games in Rust.",
        docs_pt: "Uses ECS architecture: components are structs with #[derive(Component)], systems are regular functions (fn move_player(mut query: Query<&mut Transform>, time: Res<Time>)) registered in App::new().add_plugins(DefaultPlugins). Runs in its own window or another terminal (F4).",
        key_items: "App, DefaultPlugins, Startup, Update, Commands, Query, Res, ResMut, Component, Resource, Transform, Camera2d, Sprite, ButtonInput, KeyCode, Time",
        use_line: "use bevy::prelude::*;",
        example: "#[derive(Component)]\nstruct Player;\n\nfn main() {\n    App::new()\n        .add_plugins(DefaultPlugins)\n        .add_systems(Startup, setup)\n        .add_systems(Update, move_player)\n        .run();\n}",
        doc_url: "https://docs.rs/bevy/latest/bevy/",
    },
    ExtraLibrary {
        name: "minifb",
        version: "0.27",
        category: "2D Pixel Window",
        summary: "Simple pixel-framebuffer window (Vec<u32>), ideal for learning software rendering and visual simulations.",
        docs_pt: "Creates a 2D window with Window::new and updates an RGB pixel vector (0xRRGGBB) every frame via update_with_buffer.",
        key_items: "Window, WindowOptions, Key, update_with_buffer, is_open",
        use_line: "use minifb::{Key, Window, WindowOptions};",
        example: "let mut window = Window::new(\"Demo\", 320, 200, WindowOptions::default()).unwrap();\nlet buffer = vec![0x0022_4466u32; 320 * 200];\nwhile window.is_open() && !window.is_key_down(Key::Escape) {\n    window.update_with_buffer(&buffer, 320, 200).unwrap();\n}",
        doc_url: "https://docs.rs/minifb/latest/minifb/",
    },
    ExtraLibrary {
        name: "eframe",
        version: "0.31",
        category: "Desktop GUI (egui)",
        summary: "Immediate-mode GUI framework for building native desktop windows with buttons, text inputs, sliders, and charts.",
        docs_pt: "Lets you build visual apps with buttons, text fields, and panels using egui::CentralPanel::default().show(...).",
        key_items: "egui, run_native, CentralPanel, SidePanel, Ui, heading, button, label, Slider",
        use_line: "use eframe::egui;",
        example: "egui::CentralPanel::default().show(ctx, |ui| {\n    ui.heading(\"My Rust Visual App!\");\n    if ui.button(\"Click me\").clicked() { /* ... */ }\n});",
        doc_url: "https://docs.rs/eframe/latest/eframe/",
    },
    ExtraLibrary {
        name: "tachyonfx",
        version: "0.11",
        category: "Visual Effects & Animations",
        summary: "Composable shader-like terminal effects and animations (dissolve, fade, sweep) for TUI interfaces.",
        docs_pt: "Creates smooth interpolated animations with fx::dissolve, fx::fade_from_fg, fx::sweep_in, EffectTimer, and Interpolation.",
        key_items: "fx, Effect, EffectTimer, Interpolation, dissolve, fade_from_fg, sweep_in",
        use_line: "use tachyonfx::{Effect, EffectTimer, Interpolation, fx};",
        example: "let effect = fx::fade_from_fg(ratatui::style::Color::Yellow, (600, Interpolation::SineOut));",
        doc_url: "https://docs.rs/tachyonfx/latest/tachyonfx/",
    },
    ExtraLibrary {
        name: "rand",
        version: "0.9",
        category: "Randomness & Dice Rolls",
        summary: "Generates random numbers, enemy spawns, powerups, and dice rolls in games and simulations.",
        docs_pt: "Use rand::rng().random_range(min..max) to sample numbers or random_bool(prob) for weighted events.",
        key_items: "Rng, rng, random_range, random_bool",
        use_line: "use rand::Rng;",
        example: "let mut rng = rand::rng();\nlet roll = rng.random_range(1..=10);",
        doc_url: "https://docs.rs/rand/latest/rand/",
    },
    ExtraLibrary {
        name: "rodio",
        version: "0.20",
        category: "Audio & Sound Effects",
        summary: "Plays synthesized tones, music, and sound effects in Rust games and applications.",
        docs_pt: "Opens the default audio output via OutputStream::try_default() and controls playback with Sink (e.g. SineWave for collision beeps).",
        key_items: "OutputStream, Sink, SineWave, Source",
        use_line: "use rodio::{OutputStream, Sink, source::SineWave};",
        example: "if let Ok((_stream, handle)) = OutputStream::try_default() {\n    if let Ok(sink) = Sink::try_new(&handle) { /* play sound */ }\n}",
        doc_url: "https://docs.rs/rodio/latest/rodio/",
    },
];

/// Metadados pedagógicos e de ambiente estendidos do catálogo F9 (Especificação V3 I4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct V3LibraryMetadata {
    pub system_deps_linux: &'static str,
    pub associated_v3_lessons: &'static [&'static str],
    pub common_pitfalls: &'static str,
    pub tested_environments: &'static str,
    pub review_status: &'static str,
}

impl ExtraLibrary {
    /// Retorna metadados pedagógicos, dependências de sistema Linux e status auditado.
    pub fn v3_metadata(&self) -> V3LibraryMetadata {
        self.v3_metadata_for(false)
    }

    /// Bilingual variant of `v3_metadata`.
    pub fn v3_metadata_for(&self, english: bool) -> V3LibraryMetadata {
        match self.name {
            "macroquad" => V3LibraryMetadata {
                system_deps_linux: "libx11-dev, libxi-dev, libgl1-mesa-dev, libasound2-dev",
                associated_v3_lessons: &["D07-P02", "D10-P02", "PAINT-01"],
                common_pitfalls: bi(
                    "Esquecer `next_frame().await` no loop congela a janela; usar `#[macroquad::main]` em vez de main síncrona.",
                    "Forgetting `next_frame().await` in the loop freezes the window; use `#[macroquad::main]` instead of a synchronous `main`.",
                    english,
                ),
                tested_environments: bi(
                    "Linux X11/Wayland · Windows MinGW (compilação guiada)",
                    "Linux X11/Wayland · Windows MinGW (guided build)",
                    english,
                ),
                review_status: bi(
                    "Auditada V3 · GUI depende de display gráfico ativo",
                    "V3 Audited · GUI requires an active graphical display",
                    english,
                ),
            },
            "bevy" => V3LibraryMetadata {
                system_deps_linux: "libasound2-dev, libudev-dev, libwayland-dev, libxkbcommon-dev, vulkan/mesa",
                associated_v3_lessons: &["CAVE-01", "CAVE-08", "CAVE-18"],
                common_pitfalls: bi(
                    "Compilação dev sem `[profile.dev.package.\"*\"] opt-level = 1` fica lenta; queries mutáveis concorrentes sem `Without<T>` geram erro de borrow ECS.",
                    "Dev builds without `[profile.dev.package.\"*\"] opt-level = 1` compile slowly; overlapping mutable queries without `Without<T>` cause ECS borrow errors.",
                    english,
                ),
                tested_environments: bi(
                    "Linux Vulkan/Wayland/X11 · Modo CaveGame núcleo voxel testado headless",
                    "Linux Vulkan/Wayland/X11 · CaveGame voxel core tested headless",
                    english,
                ),
                review_status: bi(
                    "Auditada V3 (0.20) · Janela 3D depende de display/GPU",
                    "V3 Audited (0.20) · 3D window requires display/GPU",
                    english,
                ),
            },
            "minifb" => V3LibraryMetadata {
                system_deps_linux: "libx11-dev, libxkbcommon-dev (X11/Wayland)",
                associated_v3_lessons: &["PAINT-04", "PAINT-07"],
                common_pitfalls: bi(
                    "Passar buffer com tamanho diferente de `largura * altura` falha em runtime; atualizar janela sempre com `is_open()`.",
                    "Passing a buffer whose length differs from `width * height` fails at runtime; keep updating the window while `is_open()`.",
                    english,
                ),
                tested_environments: bi(
                    "Linux desktop · Windows",
                    "Linux desktop · Windows",
                    english,
                ),
                review_status: bi(
                    "Auditada V3 · GUI depende de display",
                    "V3 Audited · GUI requires display",
                    english,
                ),
            },
            "eframe" => V3LibraryMetadata {
                system_deps_linux: "libxcb, libgtk-3-dev, OpenGL/glow",
                associated_v3_lessons: &["PAINT-01", "PAINT-12", "D12-P02"],
                common_pitfalls: bi(
                    "Guardar estado transitório em variáveis locais do `update()` zera tudo a cada frame; mutar apenas campos da struct do App.",
                    "Storing persistent state in local variables inside `update()` resets every frame; store state in your App struct fields instead.",
                    english,
                ),
                tested_environments: bi(
                    "Linux X11/Wayland · Windows",
                    "Linux X11/Wayland · Windows",
                    english,
                ),
                review_status: bi(
                    "Auditada V3 · Interface imediata depende de display",
                    "V3 Audited · Immediate-mode GUI requires display",
                    english,
                ),
            },
            "tachyonfx" => V3LibraryMetadata {
                system_deps_linux: bi(
                    "Nenhuma dependência nativa extra (pure Rust terminal / ratatui)",
                    "No extra native dependencies (pure Rust terminal / ratatui)",
                    english,
                ),
                associated_v3_lessons: &["F05-P01", "D10-P02"],
                common_pitfalls: bi(
                    "Chamar efeito terminado sem reiniciar `EffectTimer`; aplicar coordenadas fora da `area` do buffer.",
                    "Calling a finished effect without resetting `EffectTimer`; passing coordinates outside the buffer `area`.",
                    english,
                ),
                tested_environments: "Linux TTY/PTY · Windows Terminal · Headless CI",
                review_status: bi(
                    "Auditada e testada em headless (fx_finish.rs)",
                    "Audited and tested headless (fx_finish.rs)",
                    english,
                ),
            },
            "rand" => V3LibraryMetadata {
                system_deps_linux: bi(
                    "Nenhuma dependência de sistema (pure Rust)",
                    "No system dependencies (pure Rust)",
                    english,
                ),
                associated_v3_lessons: &["M08-P01", "D08-P02", "CAVE-02"],
                common_pitfalls: bi(
                    "Intervalo invertido `gen_range(10..5)` entra em panic; para testes determinísticos prefira sementes fixas (`SeedableRng`).",
                    "Reversed ranges like `random_range(10..5)` panic; use fixed seeds (`SeedableRng`) for deterministic tests.",
                    english,
                ),
                tested_environments: bi(
                    "Multiplataforma completa + Headless CI",
                    "Full cross-platform + Headless CI",
                    english,
                ),
                review_status: bi(
                    "Auditada e testada (Pure Rust)",
                    "Audited and tested (Pure Rust)",
                    english,
                ),
            },
            _ => V3LibraryMetadata {
                system_deps_linux: bi(
                    "libasound2-dev (ALSA) ou pipewire-alsa em Linux",
                    "libasound2-dev (ALSA) or pipewire-alsa on Linux",
                    english,
                ),
                associated_v3_lessons: &["D07-P02", "D11-P02"],
                common_pitfalls: bi(
                    "Dropar `_stream` retornada por `OutputStream::try_default()` encerra o áudio imediatamente mesmo com `Sink` vivo.",
                    "Dropping `_stream` returned by `OutputStream::try_default()` immediately stops audio even while `Sink` stays alive.",
                    english,
                ),
                tested_environments: bi(
                    "Linux ALSA/PipeWire + fallback silencioso verificado",
                    "Linux ALSA/PipeWire + verified silent fallback",
                    english,
                ),
                review_status: bi(
                    "Auditada · Fallback mudo testado automaticamente",
                    "Audited · Silent fallback automatically tested",
                    english,
                ),
            },
        }
    }
}

pub fn lookup(symbol: &str) -> Option<DocHint> {
    lookup_for(symbol, false)
}

pub fn lookup_for(symbol: &str, english: bool) -> Option<DocHint> {
    let (pt, en, url) = match symbol {
        "println" => (
            "Imprime texto formatado e adiciona uma quebra de linha.",
            "Prints formatted text followed by a newline.",
            "https://doc.rust-lang.org/std/macro.println.html",
        ),
        "print" => (
            "Imprime texto formatado sem adicionar quebra de linha.",
            "Prints formatted text without adding a newline.",
            "https://doc.rust-lang.org/std/macro.print.html",
        ),
        "format" => (
            "Monta uma String usando a sintaxe de formatação do Rust.",
            "Builds a String using Rust's formatting syntax.",
            "https://doc.rust-lang.org/std/macro.format.html",
        ),
        "String" => (
            "Texto UTF-8 que pode crescer e ser alterado.",
            "Growable, mutable UTF-8 string owned on the heap.",
            "https://doc.rust-lang.org/std/string/struct.String.html",
        ),
        "Vec" => (
            "Lista redimensionável de valores do mesmo tipo.",
            "Contiguous growable array type of homogeneous values.",
            "https://doc.rust-lang.org/std/vec/struct.Vec.html",
        ),
        "Option" => (
            "Representa um valor que pode existir (Some) ou faltar (None).",
            "Represents an optional value: either Some(value) or None.",
            "https://doc.rust-lang.org/std/option/enum.Option.html",
        ),
        "Result" => (
            "Representa sucesso (Ok) ou erro (Err) numa operação.",
            "Represents either success (Ok) or failure (Err).",
            "https://doc.rust-lang.org/std/result/enum.Result.html",
        ),
        "Some" => (
            "Variante de Option que contém um valor.",
            "Option variant containing a present value.",
            "https://doc.rust-lang.org/std/option/enum.Option.html#variant.Some",
        ),
        "None" => (
            "Variante de Option que representa ausência de valor.",
            "Option variant representing no value.",
            "https://doc.rust-lang.org/std/option/enum.Option.html#variant.None",
        ),
        "Ok" => (
            "Variante de Result que contém um resultado bem-sucedido.",
            "Result variant holding a success value.",
            "https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok",
        ),
        "Err" => (
            "Variante de Result que contém um erro.",
            "Result variant holding an error value.",
            "https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err",
        ),
        "mut" => (
            "Permite alterar uma variável ou referência após criá-la.",
            "Allows mutating a binding or reference after creation.",
            "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html",
        ),
        "fn" => (
            "Declara uma função; parâmetros e retorno têm tipos explícitos.",
            "Declares a function with explicit parameter and return types.",
            "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html",
        ),
        "let" => (
            "Declara uma variável; por padrão, ela é imutável.",
            "Declares a variable binding; immutable by default.",
            "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html",
        ),
        "main" => (
            "Função principal onde a execução do programa Rust começa.",
            "Entry-point function where Rust program execution begins.",
            "https://doc.rust-lang.org/book/ch01-02-hello-world.html",
        ),
        "eprintln" => (
            "Imprime mensagem formatada na saída de erro (stderr) com quebra de linha.",
            "Prints formatted output to standard error (stderr) with a newline.",
            "https://doc.rust-lang.org/std/macro.eprintln.html",
        ),
        "vec" => (
            "Macro para criar um Vec preenchido rapidamente (ex.: vec![1, 2, 3]).",
            "Macro to construct a populated Vec quickly (e.g. vec![1, 2, 3]).",
            "https://doc.rust-lang.org/std/macro.vec.html",
        ),
        "assert_eq" => (
            "Verifica em testes se dois valores são iguais; falha com pânico se diferirem.",
            "Test assertion verifying two values are equal; panics if they differ.",
            "https://doc.rust-lang.org/std/macro.assert_eq.html",
        ),
        "assert" => (
            "Verifica em testes se uma condição booleana é verdadeira.",
            "Test assertion ensuring a boolean expression evaluates to true.",
            "https://doc.rust-lang.org/std/macro.assert.html",
        ),
        "todo" => (
            "Marca um trecho ainda não implementado (entra em pânico se executado).",
            "Marks unimplemented code (panics when reached at runtime).",
            "https://doc.rust-lang.org/std/macro.todo.html",
        ),
        "panic" => (
            "Interrompe a thread atual exibindo uma mensagem de erro irrecuperável.",
            "Terminates the current thread with an unrecoverable error message.",
            "https://doc.rust-lang.org/std/macro.panic.html",
        ),
        "match" => (
            "Compara um valor contra padrões exaustivos e escolhe o ramo correspondente.",
            "Exhaustively compares a value against patterns and runs the matching arm.",
            "https://doc.rust-lang.org/book/ch06-02-match.html",
        ),
        "if" => (
            "Executa um ramo condicionalmente quando uma expressão bool é verdadeira.",
            "Conditionally executes a block when a boolean condition holds.",
            "https://doc.rust-lang.org/book/ch03-05-control-flow.html",
        ),
        "else" => (
            "Ramo alternativo executado quando a condição de um if é falsa.",
            "Fallback branch run when an if condition evaluates to false.",
            "https://doc.rust-lang.org/book/ch03-05-control-flow.html",
        ),
        "loop" => (
            "Laço infinito controlado explicitamente com break ou return.",
            "Infinite loop exited explicitly via break or return.",
            "https://doc.rust-lang.org/book/ch03-05-control-flow.html#repeating-code-with-loop",
        ),
        "while" => (
            "Repete um bloco enquanto uma condição booleana continuar verdadeira.",
            "Repeats a block while a boolean condition stays true.",
            "https://doc.rust-lang.org/book/ch03-05-control-flow.html#conditional-loops-with-while",
        ),
        "for" | "in" => (
            "Percorre cada elemento produzido por um iterador ou intervalo.",
            "Iterates over every item produced by an iterator or range.",
            "https://doc.rust-lang.org/book/ch03-05-control-flow.html#looping-through-a-collection-with-for",
        ),
        "break" => (
            "Interrompe imediatamente um laço (loop, while ou for).",
            "Immediately exits the innermost loop (loop, while, or for).",
            "https://doc.rust-lang.org/book/ch03-05-control-flow.html#repeating-code-with-loop",
        ),
        "continue" => (
            "Pula para a próxima iteração do laço atual.",
            "Skips straight to the next iteration of the current loop.",
            "https://doc.rust-lang.org/book/ch03-05-control-flow.html#conditional-loops-with-while",
        ),
        "struct" => (
            "Define um tipo composto agrupando campos nomeados ou em tupla.",
            "Defines a custom product type grouping named or tuple fields.",
            "https://doc.rust-lang.org/book/ch05-01-defining-structs.html",
        ),
        "enum" => (
            "Define um tipo que pode assumir uma entre várias variantes explícitas.",
            "Defines a sum type that can take one of several explicit variants.",
            "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html",
        ),
        "impl" => (
            "Implementa métodos, funções associadas ou traits para um tipo.",
            "Implements methods, associated functions, or traits on a type.",
            "https://doc.rust-lang.org/book/ch05-03-method-syntax.html",
        ),
        "trait" => (
            "Define um contrato de métodos compartilhados entre diferentes tipos.",
            "Defines shared behavior as a method contract across types.",
            "https://doc.rust-lang.org/book/ch10-02-traits.html",
        ),
        "use" => (
            "Traz módulos, tipos ou funções para o escopo atual.",
            "Brings module paths, types, or functions into the current scope.",
            "https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html",
        ),
        "mod" => (
            "Declara ou carrega um módulo para organizar código e visibilidade.",
            "Declares or loads a module to structure code and privacy.",
            "https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html",
        ),
        "pub" => (
            "Torna um item (função, campo, módulo) acessível fora do módulo atual.",
            "Makes an item (function, field, module) public outside its module.",
            "https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html",
        ),
        "return" => (
            "Encerra a função imediatamente devolvendo o valor informado.",
            "Exits the function immediately returning the specified value.",
            "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#functions-with-return-values",
        ),
        "const" | "static" => (
            "Define uma constante com tipo explícito e valor fixo durante o programa.",
            "Defines a typed constant value known throughout the program.",
            "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#constants",
        ),
        "async" | "await" => (
            "Permite operações assíncronas (como esperar próximo frame no macroquad ou I/O).",
            "Enables asynchronous operations (such as awaiting frames in macroquad or I/O).",
            "https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html",
        ),
        "move" => (
            "Transfere a posse das variáveis capturadas para dentro de uma closure ou thread.",
            "Transfers ownership of captured variables into a closure or thread.",
            "https://doc.rust-lang.org/book/ch13-01-closures.html",
        ),
        "self" | "Self" => (
            "Refere-se à instância atual (`self`) ou ao próprio tipo dentro de `impl`/`trait` (`Self`).",
            "Refers to the receiver instance (`self`) or implementing type (`Self`).",
            "https://doc.rust-lang.org/book/ch05-03-method-syntax.html",
        ),
        "str" => (
            "Fatia imutável de texto UTF-8 (&str), muito usada para emprestar texto.",
            "Immutable UTF-8 string slice (&str), commonly used for borrowed text.",
            "https://doc.rust-lang.org/std/primitive.str.html",
        ),
        "usize" => (
            "Inteiro sem sinal do tamanho de ponteiro, padrão para índices e comprimentos.",
            "Pointer-sized unsigned integer used for indexing and collection lengths.",
            "https://doc.rust-lang.org/std/primitive.usize.html",
        ),
        "i32" | "i64" => (
            "Inteiro com sinal (aceita valores negativos, zero e positivos).",
            "Signed integer type supporting negative, zero, and positive values.",
            "https://doc.rust-lang.org/std/primitive.i32.html",
        ),
        "u32" | "u64" | "u8" => (
            "Inteiro sem sinal (apenas valores não negativos, útil para pixels e contagens).",
            "Unsigned integer type for non-negative values like pixels and counts.",
            "https://doc.rust-lang.org/std/primitive.u32.html",
        ),
        "f32" | "f64" => (
            "Número decimal de ponto flutuante (muito usado em coordenadas, física e jogos).",
            "Floating-point decimal number commonly used for physics and coordinates.",
            "https://doc.rust-lang.org/std/primitive.f64.html",
        ),
        "bool" => (
            "Tipo booleano com exatamente dois valores possíveis: true ou false.",
            "Boolean primitive type with only two values: true or false.",
            "https://doc.rust-lang.org/std/primitive.bool.html",
        ),
        "char" => (
            "Caractere escalar Unicode de 4 bytes escrito entre aspas simples ('a').",
            "4-byte Unicode scalar character written in single quotes ('a').",
            "https://doc.rust-lang.org/std/primitive.char.html",
        ),
        "HashMap" => (
            "Mapa associativo chave → valor em std::collections::HashMap.",
            "Hash-based key-value associative map in std::collections::HashMap.",
            "https://doc.rust-lang.org/std/collections/struct.HashMap.html",
        ),
        "Clone" | "clone" => (
            "Trait e método para duplicar explicitamente um valor em memória.",
            "Trait and method for explicitly duplicating a value.",
            "https://doc.rust-lang.org/std/clone/trait.Clone.html",
        ),
        "Debug" | "derive" => (
            "Implementa automaticamente traits como Debug/Clone via #[derive(...)].",
            "Automatically generates trait implementations like Debug/Clone via #[derive(...)].",
            "https://doc.rust-lang.org/std/fmt/trait.Debug.html",
        ),
        "cfg" | "test" => (
            "Atributo condicional de compilação (ex.: #[cfg(test)] e #[test] para testes).",
            "Conditional compilation attribute (e.g. #[cfg(test)] and #[test]).",
            "https://doc.rust-lang.org/book/ch11-01-writing-tests.html",
        ),
        "Iterator" | "iter" => (
            "Percorre uma sequência preguiçosamente; combina com map, filter e collect.",
            "Lazily traverses a sequence; composes cleanly with map, filter, and collect.",
            "https://doc.rust-lang.org/std/iter/trait.Iterator.html",
        ),
        "collect" => (
            "Consome um iterador transformando seus itens em Vec, String ou outra coleção.",
            "Consumes an iterator to build a Vec, String, or other collection.",
            "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect",
        ),
        "map" => (
            "Transforma cada elemento de um iterador (ou Option/Result) com uma closure.",
            "Transforms each element of an iterator (or Option/Result) with a closure.",
            "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map",
        ),
        "filter" => (
            "Mantém apenas os elementos de um iterador que satisfazem um predicado.",
            "Keeps only iterator elements that satisfy a predicate.",
            "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter",
        ),
        "len" => (
            "Retorna a quantidade de elementos (ou bytes no caso de str/String).",
            "Returns the number of elements (or bytes for str/String).",
            "https://doc.rust-lang.org/std/vec/struct.Vec.html#method.len",
        ),
        "is_empty" => (
            "Retorna true se a coleção ou texto não possuir nenhum elemento.",
            "Returns true if the collection or string has length zero.",
            "https://doc.rust-lang.org/std/vec/struct.Vec.html#method.is_empty",
        ),
        "push" => (
            "Adiciona um novo elemento ao final de um Vec ou caractere em uma String.",
            "Appends an element to a Vec or a character to a String.",
            "https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push",
        ),
        "lines" => (
            "Retorna um iterador sobre cada linha de texto.",
            "Returns an iterator over each line in a string slice.",
            "https://doc.rust-lang.org/std/primitive.str.html#method.lines",
        ),
        "trim" => (
            "Remove espaços e quebras de linha do início e do fim de um &str.",
            "Removes leading and trailing whitespace from a string slice.",
            "https://doc.rust-lang.org/std/primitive.str.html#method.trim",
        ),
        "parse" => (
            "Converte texto em outro tipo (ex.: texto.parse::<i32>()) devolvendo Result.",
            "Parses a string slice into another type (e.g. text.parse::<i32>()), returning Result.",
            "https://doc.rust-lang.org/std/primitive.str.html#method.parse",
        ),
        "to_string" => (
            "Converte o valor ou &str em uma String com posse.",
            "Converts a Display value or &str into an owned String.",
            "https://doc.rust-lang.org/std/string/trait.ToString.html",
        ),
        "as_str" => (
            "Empresta uma String como fatia &str sem alocar memória nova.",
            "Borrows a String as a &str slice without allocating.",
            "https://doc.rust-lang.org/std/string/struct.String.html#method.as_str",
        ),
        "unwrap" => (
            "Extrai Some/Ok diretamente, mas entra em pânico se for None/Err.",
            "Unwraps Some/Ok directly, panicking if the value is None/Err.",
            "https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap",
        ),
        "expect" => (
            "Como unwrap(), mas permite informar uma mensagem explicando o motivo em caso de erro.",
            "Like unwrap(), but lets you attach a contextual panic message.",
            "https://doc.rust-lang.org/std/option/enum.Option.html#method.expect",
        ),
        "unwrap_or" => (
            "Extrai o valor de Option/Result ou retorna um valor padrão seguro.",
            "Extracts the Option/Result inner value or falls back to a safe default.",
            "https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or",
        ),
        // --- Bibliotecas extras: macroquad ---
        "macroquad" => (
            "Mini engine de jogos 2D em Rust: janela gráfica, formas, sprite, teclado e loop async.",
            "2D mini game engine in Rust: windowing, shapes, sprites, keyboard input, and async loop.",
            "https://docs.rs/macroquad/latest/macroquad/",
        ),
        "clear_background" => (
            "macroquad: preenche toda a tela com uma cor (chame no início de cada frame).",
            "macroquad: clears the entire screen with a color at the start of each frame.",
            "https://docs.rs/macroquad/latest/macroquad/window/fn.clear_background.html",
        ),
        "draw_circle" => (
            "macroquad: desenha um círculo preenchido na posição (x, y) com raio r e cor.",
            "macroquad: draws a filled circle at (x, y) with radius r and color.",
            "https://docs.rs/macroquad/latest/macroquad/shapes/fn.draw_circle.html",
        ),
        "draw_rectangle" => (
            "macroquad: desenha um retângulo preenchido em (x, y, largura, altura, cor).",
            "macroquad: draws a filled rectangle at (x, y, width, height, color).",
            "https://docs.rs/macroquad/latest/macroquad/shapes/fn.draw_rectangle.html",
        ),
        "draw_line" => (
            "macroquad: desenha um segmento de reta entre (x1, y1) e (x2, y2) com espessura e cor.",
            "macroquad: draws a line segment between (x1, y1) and (x2, y2) with thickness and color.",
            "https://docs.rs/macroquad/latest/macroquad/shapes/fn.draw_line.html",
        ),
        "draw_text" => (
            "macroquad: escreve texto na tela em (x, y) com tamanho de fonte e cor.",
            "macroquad: renders text on screen at (x, y) with font size and color.",
            "https://docs.rs/macroquad/latest/macroquad/text/fn.draw_text.html",
        ),
        "is_key_down" => (
            "macroquad: retorna true enquanto a tecla especificada estiver pressionada.",
            "macroquad: returns true as long as the given key is held down.",
            "https://docs.rs/macroquad/latest/macroquad/input/fn.is_key_down.html",
        ),
        "is_key_pressed" => (
            "macroquad: retorna true apenas no frame em que a tecla acabou de ser apertada.",
            "macroquad: returns true only on the frame the key was freshly pressed.",
            "https://docs.rs/macroquad/latest/macroquad/input/fn.is_key_pressed.html",
        ),
        "mouse_position" => (
            "macroquad: devolve a tupla (x, y) atual do ponteiro do mouse na janela.",
            "macroquad: returns the mouse pointer's current (x, y) position in the window.",
            "https://docs.rs/macroquad/latest/macroquad/input/fn.mouse_position.html",
        ),
        "screen_width" => (
            "macroquad: largura atual da janela do jogo em pixels (f32).",
            "macroquad: current window width in pixels (f32).",
            "https://docs.rs/macroquad/latest/macroquad/window/fn.screen_width.html",
        ),
        "screen_height" => (
            "macroquad: altura atual da janela do jogo em pixels (f32).",
            "macroquad: current window height in pixels (f32).",
            "https://docs.rs/macroquad/latest/macroquad/window/fn.screen_height.html",
        ),
        "next_frame" => (
            "macroquad: apresenta o quadro desenhado na janela e espera o próximo ciclo (.await).",
            "macroquad: presents the drawn frame and awaits the next tick (.await).",
            "https://docs.rs/macroquad/latest/macroquad/window/fn.next_frame.html",
        ),
        "get_frame_time" => (
            "macroquad: tempo delta (em segundos) desde o último frame — útil para movimentação suave.",
            "macroquad: frame delta time in seconds — great for frame-rate-independent motion.",
            "https://docs.rs/macroquad/latest/macroquad/time/fn.get_frame_time.html",
        ),
        "Color" => (
            "macroquad: representa cor RGBA para formas, fundo e textos (ex.: RED, GOLD, Color::new).",
            "macroquad: RGBA color value for shapes, backgrounds, and text (e.g. RED, GOLD, Color::new).",
            "https://docs.rs/macroquad/latest/macroquad/color/struct.Color.html",
        ),
        "KeyCode" => (
            "macroquad: enumeração das teclas do teclado (KeyCode::Up, KeyCode::Space, etc.).",
            "macroquad: keyboard key enum (KeyCode::Up, KeyCode::Space, etc.).",
            "https://docs.rs/macroquad/latest/macroquad/input/enum.KeyCode.html",
        ),
        "Conf" => (
            "macroquad: configurações iniciais da janela (título, largura, altura).",
            "macroquad: initial window configuration (title, width, height).",
            "https://docs.rs/macroquad/latest/macroquad/conf/struct.Conf.html",
        ),
        "vec2" | "Vec2" => (
            "macroquad/glam: vetor 2D (x, y) usado para posições e velocidades em jogos.",
            "macroquad/glam: 2D vector (x, y) used for positions and velocities in games.",
            "https://docs.rs/macroquad/latest/macroquad/math/struct.Vec2.html",
        ),
        // --- minifb ---
        "minifb" => (
            "Biblioteca mínima de janela gráfica por buffer de pixels RGB 2D.",
            "Minimal 2D window library backed by a raw RGB pixel buffer.",
            "https://docs.rs/minifb/latest/minifb/",
        ),
        "Window" => (
            "minifb: representa a janela gráfica aberta para desenho direto de pixels.",
            "minifb: represents an open graphical window for direct pixel drawing.",
            "https://docs.rs/minifb/latest/minifb/struct.Window.html",
        ),
        "WindowOptions" => (
            "minifb: opções da janela (redimensionamento, título, escala).",
            "minifb: window creation options (resize, title, scaling).",
            "https://docs.rs/minifb/latest/minifb/struct.WindowOptions.html",
        ),
        "Key" => (
            "minifb: teclas suportadas em `janela.is_key_down(Key::Escape)`.",
            "minifb: key enum used with `window.is_key_down(Key::Escape)`.",
            "https://docs.rs/minifb/latest/minifb/enum.Key.html",
        ),
        "update_with_buffer" => (
            "minifb: envia um slice &[u32] de cores RGB para redesenhar toda a janela.",
            "minifb: sends a &[u32] RGB buffer slice to redraw the window.",
            "https://docs.rs/minifb/latest/minifb/struct.Window.html#method.update_with_buffer",
        ),
        "is_open" => (
            "minifb: retorna true enquanto a janela ainda não foi fechada pelo usuário.",
            "minifb: returns true while the window has not been closed by the user.",
            "https://docs.rs/minifb/latest/minifb/struct.Window.html#method.is_open",
        ),
        // --- eframe / egui ---
        "eframe" => (
            "Runner oficial desktop/web para interfaces gráficas imediatas com egui.",
            "Official desktop/web app runner for egui immediate-mode GUIs.",
            "https://docs.rs/eframe/latest/eframe/",
        ),
        "egui" => (
            "Biblioteca GUI imediata com botões, caixas de texto, sliders e layouts.",
            "Immediate-mode GUI library with buttons, text boxes, sliders, and layouts.",
            "https://docs.rs/egui/latest/egui/",
        ),
        "run_native" => (
            "eframe: abre a janela nativa executando a aplicação egui.",
            "eframe: launches the native OS window hosting the egui application.",
            "https://docs.rs/eframe/latest/eframe/fn.run_native.html",
        ),
        "CentralPanel" => (
            "egui: área central responsiva onde ficam os controles principais da janela.",
            "egui: responsive central area containing the window's main UI.",
            "https://docs.rs/egui/latest/egui/struct.CentralPanel.html",
        ),
        "SidePanel" => (
            "egui: painel lateral esquerdo/direito para menus e ferramentas.",
            "egui: left/right side panel for navigation menus and tools.",
            "https://docs.rs/egui/latest/egui/struct.SidePanel.html",
        ),
        "Ui" => (
            "egui: construtor imediato de widgets (`ui.heading`, `ui.button`, `ui.label`).",
            "egui: immediate-mode widget builder (`ui.heading`, `ui.button`, `ui.label`).",
            "https://docs.rs/egui/latest/egui/struct.Ui.html",
        ),
        "heading" | "button" | "label" | "Slider" => (
            "egui: componentes visuais básicos (título, botão clicável, rótulo e controle deslizante).",
            "egui: core visual widgets (heading, clickable button, label, and slider).",
            "https://docs.rs/egui/latest/egui/struct.Ui.html#method.button",
        ),
        // --- tachyonfx ---
        "tachyonfx" => (
            "Biblioteca de efeitos visuais e animações por célula no terminal (fade, dissolve, sweep).",
            "Per-cell terminal visual effects and animation library (fade, dissolve, sweep).",
            "https://docs.rs/tachyonfx/latest/tachyonfx/",
        ),
        "fx" | "dissolve" | "fade_from_fg" | "sweep_in" => (
            "tachyonfx: funções de fábrica para criar efeitos visuais animados composáveis.",
            "tachyonfx: factory helpers for composable animated terminal effects.",
            "https://docs.rs/tachyonfx/latest/tachyonfx/fx/index.html",
        ),
        "Effect" | "EffectTimer" | "Interpolation" => (
            "tachyonfx: representa a duração, curva de suavização e execução de uma animação.",
            "tachyonfx: represents duration, easing curve, and playback of an animation.",
            "https://docs.rs/tachyonfx/latest/tachyonfx/struct.Effect.html",
        ),
        // --- rand ---
        "rand" => (
            "Gerador de números aleatórios padrão da comunidade Rust.",
            "De-facto standard random number generator library in Rust.",
            "https://docs.rs/rand/latest/rand/",
        ),
        "Rng" | "rng" | "random_range" | "random_bool" => (
            "rand: trait e métodos para sortear intervalos (`rng.random_range(1..=10)`) e booleanos.",
            "rand: trait and helpers for sampling ranges (`rng.random_range(1..=10)`) and booleans.",
            "https://docs.rs/rand/latest/rand/trait.Rng.html",
        ),
        // --- rodio ---
        "rodio" => (
            "Biblioteca de reprodução e síntese simples de áudio em Rust.",
            "Audio playback and simple tone synthesis library in Rust.",
            "https://docs.rs/rodio/latest/rodio/",
        ),
        "OutputStream" | "Sink" | "SineWave" | "Source" => (
            "rodio: tipos principais para abrir áudio, controlar fila de reprodução e gerar ondas sonoras.",
            "rodio: core types for opening audio output, managing playback sinks, and generating waveforms.",
            "https://docs.rs/rodio/latest/rodio/struct.Sink.html",
        ),
        // --- bevy (Engine 2D/3D ECS) ---
        "bevy" => (
            "Engine de jogos 2D/3D em Rust baseada em arquitetura ECS (Entities, Components, Systems).",
            "2D/3D game engine in Rust built on an ECS (Entities, Components, Systems) architecture.",
            "https://docs.rs/bevy/latest/bevy/",
        ),
        "App" | "add_plugins" | "add_systems" => (
            "bevy: núcleo da aplicação (`App::new().add_plugins(DefaultPlugins).add_systems(Update, fn_sistema).run()`).",
            "bevy: application builder (`App::new().add_plugins(DefaultPlugins).add_systems(Update, my_system).run()`).",
            "https://docs.rs/bevy/latest/bevy/app/struct.App.html",
        ),
        "DefaultPlugins" | "MinimalPlugins" | "Plugin" => (
            "bevy: conjunto padrão de subsistemas (janela, renderizador, entrada, áudio e tempo).",
            "bevy: standard engine subsystem bundle (windowing, rendering, input, audio, time).",
            "https://docs.rs/bevy/latest/bevy/struct.DefaultPlugins.html",
        ),
        "Startup" | "Update" | "FixedUpdate" | "PostUpdate" => (
            "bevy: agendas (Schedules) — Startup roda uma vez ao iniciar; Update roda todo frame.",
            "bevy: execution Schedules — Startup runs once on boot; Update runs every frame.",
            "https://docs.rs/bevy/latest/bevy/app/struct.Startup.html",
        ),
        "Commands" | "spawn" | "despawn" => (
            "bevy ECS: enfileira comandos estruturais seguros no mundo (criar/remover entidades e componentes).",
            "bevy ECS: queues safe structural world mutations (spawning/despawning entities and components).",
            "https://docs.rs/bevy/latest/bevy/ecs/system/struct.Commands.html",
        ),
        "Query" | "With" | "Without" | "Changed" | "Added" => (
            "bevy ECS: consulta e itera rapidamente todas as entidades que possuem certos Componentes.",
            "bevy ECS: queries and iterates all entities matching a set of Components.",
            "https://docs.rs/bevy/latest/bevy/ecs/system/struct.Query.html",
        ),
        "Res" | "ResMut" | "Resource" => (
            "bevy ECS: acesso somente leitura (`Res<T>`) ou mutável (`ResMut<T>`) a recursos globais do jogo.",
            "bevy ECS: read-only (`Res<T>`) or mutable (`ResMut<T>`) access to global game resources.",
            "https://docs.rs/bevy/latest/bevy/ecs/system/struct.Res.html",
        ),
        "Local" => (
            "bevy ECS: estado privado persistente exclusivo de um único sistema entre frames.",
            "bevy ECS: private per-system state persisted across frames.",
            "https://docs.rs/bevy/latest/bevy/ecs/system/struct.Local.html",
        ),
        "Component" | "Bundle" | "Entity" => (
            "bevy ECS: `#[derive(Component)]` transforma structs/enums em dados anexáveis a entidades (`Entity`).",
            "bevy ECS: `#[derive(Component)]` turns structs/enums into attachable Entity data.",
            "https://docs.rs/bevy/latest/bevy/ecs/component/trait.Component.html",
        ),
        "Event" | "EventReader" | "EventWriter" => (
            "bevy ECS: barramento tipado de eventos entre sistemas sem acoplamento direto.",
            "bevy ECS: typed event bus for decoupled system-to-system messaging.",
            "https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html",
        ),
        "Transform" | "GlobalTransform" | "Vec3" | "Quat" => (
            "bevy: posição, rotação e escala 2D/3D de uma entidade (`Transform::from_xyz(x, y, z)`).",
            "bevy: 2D/3D translation, rotation, and scale (`Transform::from_xyz(x, y, z)`).",
            "https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html",
        ),
        "Camera2d" | "Camera3d" => (
            "bevy: cria uma câmera 2D ou 3D no mundo (`commands.spawn(Camera2d);`).",
            "bevy: spawns a 2D or 3D camera in the world (`commands.spawn(Camera2d);`).",
            "https://docs.rs/bevy/latest/bevy/core_pipeline/core_2d/struct.Camera2d.html",
        ),
        "Sprite" | "Text2d" | "Gizmos" => (
            "bevy: componentes de renderização 2D (sprites retangulares, textos 2D e linhas/formas de debug).",
            "bevy: 2D rendering components (rectangular sprites, 2D text, debug shapes/lines).",
            "https://docs.rs/bevy/latest/bevy/sprite/struct.Sprite.html",
        ),
        "Time" | "Timer" | "delta_secs" => (
            "bevy: relógio do jogo (`tempo.delta_secs()`) para movimentação suave independente de FPS.",
            "bevy: game clock (`time.delta_secs()`) for smooth FPS-independent movement.",
            "https://docs.rs/bevy/latest/bevy/time/struct.Time.html",
        ),
        "ButtonInput" | "just_pressed" | "pressed" => (
            "bevy: estado de teclado/mouse via recurso (`teclas: Res<ButtonInput<KeyCode>>`).",
            "bevy: keyboard/mouse input state resource (`keys: Res<ButtonInput<KeyCode>>`).",
            "https://docs.rs/bevy/latest/bevy/input/struct.ButtonInput.html",
        ),
        "AssetServer" | "Handle" | "Image" => (
            "bevy: carregamento assíncrono de imagens, fontes e sons da pasta `assets/`.",
            "bevy: asynchronous loader for images, fonts, and audio from `assets/`.",
            "https://docs.rs/bevy/latest/bevy/asset/struct.AssetServer.html",
        ),
        _ => return None,
    };
    Some(DocHint {
        summary: bi(pt, en, english),
        url,
    })
}

/// URL da tradução em português do Rust Book quando houver tópico equivalente.
pub fn lookup_pt_url(symbol: &str) -> Option<&'static str> {
    let url = match symbol {
        "main" => "https://rust-br.github.io/rust-book-pt-br/ch01-02-hello-world.html",
        "let" | "mut" | "const" | "static" => {
            "https://rust-br.github.io/rust-book-pt-br/ch03-01-variables-and-mutability.html"
        }
        "str" | "usize" | "i32" | "i64" | "u32" | "u64" | "u8" | "f32" | "f64" | "bool"
        | "char" | "String" => "https://rust-br.github.io/rust-book-pt-br/ch03-02-data-types.html",
        "fn" | "return" => {
            "https://rust-br.github.io/rust-book-pt-br/ch03-03-how-functions-work.html"
        }
        "if" | "else" | "loop" | "while" | "for" | "in" | "break" | "continue" => {
            "https://rust-br.github.io/rust-book-pt-br/ch03-05-control-flow.html"
        }
        "struct" => "https://rust-br.github.io/rust-book-pt-br/ch05-01-defining-structs.html",
        "impl" | "self" | "Self" => {
            "https://rust-br.github.io/rust-book-pt-br/ch05-03-method-syntax.html"
        }
        "enum" | "Option" | "Some" | "None" => {
            "https://rust-br.github.io/rust-book-pt-br/ch06-01-defining-an-enum.html"
        }
        "match" => "https://rust-br.github.io/rust-book-pt-br/ch06-02-match.html",
        "mod" => {
            "https://rust-br.github.io/rust-book-pt-br/ch07-02-defining-modules-to-control-scope-and-privacy.html"
        }
        "pub" => {
            "https://rust-br.github.io/rust-book-pt-br/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html"
        }
        "use" => {
            "https://rust-br.github.io/rust-book-pt-br/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html"
        }
        "Vec" | "vec" | "push" | "len" | "is_empty" => {
            "https://rust-br.github.io/rust-book-pt-br/ch08-01-vectors.html"
        }
        "HashMap" => "https://rust-br.github.io/rust-book-pt-br/ch08-03-hash-maps.html",
        "Result" | "Ok" | "Err" | "unwrap" | "expect" | "unwrap_or" | "panic" => {
            "https://rust-br.github.io/rust-book-pt-br/ch09-02-recoverable-errors-with-result.html"
        }
        "trait" | "Clone" | "clone" | "Debug" | "derive" => {
            "https://rust-br.github.io/rust-book-pt-br/ch10-02-traits.html"
        }
        "test" | "cfg" | "assert" | "assert_eq" => {
            "https://rust-br.github.io/rust-book-pt-br/ch11-01-writing-tests.html"
        }
        "Iterator" | "iter" | "map" | "filter" | "collect" | "move" => {
            "https://rust-br.github.io/rust-book-pt-br/ch13-02-iterators.html"
        }
        "async" | "await" => "https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html",
        _ => return None,
    };
    Some(url)
}

/// Retorna a URL preferida (português quando disponível, senão inglês oficial).
pub fn preferred_doc_url(symbol: &str) -> Option<&'static str> {
    preferred_doc_url_for(symbol, false)
}

pub fn preferred_doc_url_for(symbol: &str, english: bool) -> Option<&'static str> {
    if english {
        lookup_for(symbol, true).map(|h| h.url)
    } else {
        lookup_pt_url(symbol).or_else(|| lookup_for(symbol, false).map(|h| h.url))
    }
}

/// Resolve documentação para um símbolo sob o cursor/ponteiro: primeiro busca no
/// glossário Rust/bibliotecas extras; caso contrário, inspeciona declarações locais
/// (`let`, `let mut`, `fn`, `struct`, `enum`, `const`, parâmetros) no código aberto.
pub fn resolve_symbol_doc(symbol: &str, code: &str) -> Option<ResolvedDoc> {
    resolve_symbol_doc_for(symbol, code, false)
}

pub fn resolve_symbol_doc_for(symbol: &str, code: &str, english: bool) -> Option<ResolvedDoc> {
    if let Some(hint) = lookup_for(symbol, english) {
        return Some(ResolvedDoc {
            title: symbol.to_string(),
            summary: hint.summary.to_string(),
            url_pt: if english {
                None
            } else {
                lookup_pt_url(symbol).map(str::to_string)
            },
            url_en: hint.url.to_string(),
        });
    }

    for raw_line in code.lines() {
        let trimmed = raw_line.split("//").next().unwrap_or("").trim();
        if trimmed.is_empty() {
            continue;
        }
        if contains_ident_after(trimmed, "let mut ", symbol) {
            return Some(ResolvedDoc {
                title: format!(
                    "{symbol} ({})",
                    bi("variável mutável", "mutable variable", english)
                ),
                summary: if english {
                    format!(
                        "Local mutable variable (`let mut {symbol}`) defined in your code. Its value can change during execution."
                    )
                } else {
                    format!(
                        "Variável mutável local (`let mut {symbol}`) definida no seu código. Seu valor pode mudar durante a execução."
                    )
                },
                url_pt: (!english).then(|| {
                    "https://rust-br.github.io/rust-book-pt-br/ch03-01-variables-and-mutability.html"
                        .to_string()
                }),
                url_en: "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html"
                    .to_string(),
            });
        }
        if contains_ident_after(trimmed, "let ", symbol) {
            return Some(ResolvedDoc {
                title: format!("{symbol} ({})", bi("variável", "variable", english)),
                summary: if english {
                    format!(
                        "Local immutable variable (`let {symbol}`) defined in your code. Variables in Rust are immutable by default."
                    )
                } else {
                    format!(
                        "Variável imutável local (`let {symbol}`) definida no seu código. Em Rust, variáveis são imutáveis por padrão."
                    )
                },
                url_pt: (!english).then(|| {
                    "https://rust-br.github.io/rust-book-pt-br/ch03-01-variables-and-mutability.html"
                        .to_string()
                }),
                url_en: "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html"
                    .to_string(),
            });
        }
        if contains_ident_after(trimmed, "fn ", symbol)
            || contains_ident_after(trimmed, "pub fn ", symbol)
            || contains_ident_after(trimmed, "async fn ", symbol)
            || contains_ident_after(trimmed, "pub async fn ", symbol)
        {
            return Some(ResolvedDoc {
                title: format!("fn {symbol}"),
                summary: if english {
                    format!(
                        "Function `{symbol}` defined in your project to organize and reuse a block of logic."
                    )
                } else {
                    format!(
                        "Função `{symbol}` definida no seu projeto para organizar e reutilizar um bloco de lógica."
                    )
                },
                url_pt: (!english).then(|| {
                    "https://rust-br.github.io/rust-book-pt-br/ch03-03-how-functions-work.html"
                        .to_string()
                }),
                url_en: "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html"
                    .to_string(),
            });
        }
        if contains_ident_after(trimmed, "struct ", symbol)
            || contains_ident_after(trimmed, "pub struct ", symbol)
        {
            return Some(ResolvedDoc {
                title: format!("struct {symbol}"),
                summary: if english {
                    format!(
                        "Struct `{symbol}` created in your project to group related fields together."
                    )
                } else {
                    format!(
                        "Estrutura `{symbol}` criada no seu projeto para agrupar campos relacionados."
                    )
                },
                url_pt: (!english).then(|| {
                    "https://rust-br.github.io/rust-book-pt-br/ch05-01-defining-structs.html"
                        .to_string()
                }),
                url_en: "https://doc.rust-lang.org/book/ch05-01-defining-structs.html".to_string(),
            });
        }
        if contains_ident_after(trimmed, "enum ", symbol)
            || contains_ident_after(trimmed, "pub enum ", symbol)
        {
            return Some(ResolvedDoc {
                title: format!("enum {symbol}"),
                summary: if english {
                    format!(
                        "Enum `{symbol}` defined in your project representing possible states or variants."
                    )
                } else {
                    format!(
                        "Enumeração `{symbol}` definida no seu projeto representando estados ou variantes possíveis."
                    )
                },
                url_pt: (!english).then(|| {
                    "https://rust-br.github.io/rust-book-pt-br/ch06-01-defining-an-enum.html"
                        .to_string()
                }),
                url_en: "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html".to_string(),
            });
        }
        if contains_ident_after(trimmed, "const ", symbol)
            || contains_ident_after(trimmed, "pub const ", symbol)
            || contains_ident_after(trimmed, "static ", symbol)
        {
            return Some(ResolvedDoc {
                title: format!("const {symbol}"),
                summary: if english {
                    format!("Constant `{symbol}` defined in your code.")
                } else {
                    format!("Constante `{symbol}` definida no seu código.")
                },
                url_pt: (!english).then(|| {
                    "https://rust-br.github.io/rust-book-pt-br/ch03-01-variables-and-mutability.html"
                        .to_string()
                }),
                url_en: "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html"
                    .to_string(),
            });
        }
        if trimmed.contains("fn ") && trimmed.contains(&format!("{symbol}:")) {
            return Some(ResolvedDoc {
                title: format!("{symbol} ({})", bi("parâmetro", "parameter", english)),
                summary: if english {
                    format!("Parameter `{symbol}` accepted by the function in your code.")
                } else {
                    format!("Parâmetro `{symbol}` recebido pela função no seu código.")
                },
                url_pt: (!english).then(|| {
                    "https://rust-br.github.io/rust-book-pt-br/ch03-03-how-functions-work.html"
                        .to_string()
                }),
                url_en: "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html"
                    .to_string(),
            });
        }
    }
    None
}

fn contains_ident_after(line: &str, keyword_prefix: &str, symbol: &str) -> bool {
    let Some(pos) = line.find(keyword_prefix) else {
        return false;
    };
    let rest = line[pos + keyword_prefix.len()..].trim_start();
    let Some(after) = rest.strip_prefix(symbol) else {
        return false;
    };
    !after
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Insere `use ...;` no topo correto do arquivo Rust (após `//!`, `#![...]` e `use` existentes)
/// sem duplicar caso já esteja presente.
pub fn insert_use_line_in_rust_code(code: &str, use_line: &str) -> String {
    let target = use_line.trim();
    if target.is_empty() || code.lines().any(|l| l.trim() == target) {
        return code.to_string();
    }
    let lines: Vec<&str> = code.lines().collect();
    let mut insert_idx = 0usize;
    let mut saw_use = false;
    for (idx, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("//!") || t.starts_with("#![") {
            insert_idx = idx + 1;
        } else if t.starts_with("use ") {
            insert_idx = idx + 1;
            saw_use = true;
        } else if t.is_empty() && !saw_use && idx == insert_idx {
            insert_idx = idx + 1;
        } else {
            break;
        }
    }
    let mut out = String::new();
    for (idx, line) in lines.iter().enumerate() {
        if idx == insert_idx {
            out.push_str(target);
            out.push('\n');
        }
        out.push_str(line);
        out.push('\n');
    }
    if insert_idx >= lines.len() {
        out.push_str(target);
        out.push('\n');
    }
    out
}

/// Garante que `name = "version"` exista em `[dependencies]` no `Cargo.toml`.
pub fn ensure_dependency_in_cargo_toml(cargo_toml: &str, name: &str, version: &str) -> String {
    let mut in_deps = false;
    let mut deps_header_idx = None;
    let lines: Vec<&str> = cargo_toml.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            if t == "[dependencies]" {
                in_deps = true;
                deps_header_idx = Some(idx);
            } else if t == format!("[dependencies.{name}]") {
                return cargo_toml.to_string();
            } else {
                in_deps = false;
            }
            continue;
        }
        if in_deps
            && let Some(after) = t.strip_prefix(name)
            && after.trim_start().starts_with('=')
        {
            return cargo_toml.to_string();
        }
    }
    let dep_line = format!("{name} = \"{version}\"");
    if let Some(header_idx) = deps_header_idx {
        let mut out = String::new();
        for (idx, line) in lines.iter().enumerate() {
            out.push_str(line);
            out.push('\n');
            if idx == header_idx {
                out.push_str(&dep_line);
                out.push('\n');
            }
        }
        out
    } else {
        let mut out = cargo_toml.trim_end().to_string();
        out.push_str("\n\n[dependencies]\n");
        out.push_str(&dep_line);
        out.push('\n');
        out
    }
}

/// Adiciona `[profile.dev]` com otimizações recomendadas pelo Bevy (`opt-level = 1`
/// no crate do jogo e `3` nas dependências) para compilar e rodar fluido no modo debug.
pub fn ensure_bevy_fast_dev_profile_in_cargo_toml(cargo_toml: &str) -> String {
    if cargo_toml.contains("[profile.dev]") {
        return cargo_toml.to_string();
    }
    let mut out = cargo_toml.trim_end().to_string();
    out.push_str(
        "\n\n# Otimizações recomendadas pelo Bevy para rodar rápido no modo dev:\n[profile.dev]\nopt-level = 1\n\n[profile.dev.package.\"*\"]\nopt-level = 3\n",
    );
    out
}

/// Código inicial Bevy 0.20 (ECS 2D) comentado para estudantes.
pub fn bevy_starter_template() -> &'static str {
    bevy_starter_template_for(false)
}

/// Bilingual Bevy 0.20 2D ECS starter template.
pub fn bevy_starter_template_for(english: bool) -> &'static str {
    if english {
        return concat!(
            "use bevy::prelude::*;\n\n",
            "/// Component marking the player entity and storing its speed in pixels/s.\n",
            "#[derive(Component)]\n",
            "struct Player {\n",
            "    speed: f32,\n",
            "}\n\n",
            "fn main() {\n",
            "    App::new()\n",
            "        .add_plugins(DefaultPlugins)\n",
            "        .add_systems(Startup, setup_scene)\n",
            "        .add_systems(Update, move_player)\n",
            "        .run();\n",
            "}\n\n",
            "fn setup_scene(mut commands: Commands) {\n",
            "    commands.spawn(Camera2d);\n",
            "    commands.spawn((\n",
            "        Sprite::from_color(Color::srgb(0.25, 0.75, 0.95), Vec2::new(48.0, 48.0)),\n",
            "        Transform::from_xyz(0.0, 0.0, 0.0),\n",
            "        Player { speed: 260.0 },\n",
            "    ));\n",
            "}\n\n",
            "fn move_player(\n",
            "    keys: Res<ButtonInput<KeyCode>>,\n",
            "    time: Res<Time>,\n",
            "    mut query: Query<(&mut Transform, &Player)>,\n",
            ") {\n",
            "    let mut direction = Vec2::ZERO;\n",
            "    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {\n",
            "        direction.x -= 1.0;\n",
            "    }\n",
            "    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {\n",
            "        direction.x += 1.0;\n",
            "    }\n",
            "    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {\n",
            "        direction.y += 1.0;\n",
            "    }\n",
            "    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {\n",
            "        direction.y -= 1.0;\n",
            "    }\n\n",
            "    if direction != Vec2::ZERO {\n",
            "        let step = direction.normalize() * time.delta_secs();\n",
            "        for (mut transform, player) in &mut query {\n",
            "            transform.translation.x += step.x * player.speed;\n",
            "            transform.translation.y += step.y * player.speed;\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
    }
    concat!(
        "use bevy::prelude::*;\n\n",
        "/// Componente que marca a entidade do jogador e guarda sua velocidade em pixels/s.\n",
        "#[derive(Component)]\n",
        "struct Jogador {\n",
        "    velocidade: f32,\n",
        "}\n\n",
        "fn main() {\n",
        "    App::new()\n",
        "        .add_plugins(DefaultPlugins)\n",
        "        .add_systems(Startup, configurar_cena)\n",
        "        .add_systems(Update, mover_jogador)\n",
        "        .run();\n",
        "}\n\n",
        "fn configurar_cena(mut commands: Commands) {\n",
        "    commands.spawn(Camera2d);\n",
        "    commands.spawn((\n",
        "        Sprite::from_color(Color::srgb(0.25, 0.75, 0.95), Vec2::new(48.0, 48.0)),\n",
        "        Transform::from_xyz(0.0, 0.0, 0.0),\n",
        "        Jogador { velocidade: 260.0 },\n",
        "    ));\n",
        "}\n\n",
        "fn mover_jogador(\n",
        "    teclas: Res<ButtonInput<KeyCode>>,\n",
        "    tempo: Res<Time>,\n",
        "    mut query: Query<(&mut Transform, &Jogador)>,\n",
        ") {\n",
        "    let mut direcao = Vec2::ZERO;\n",
        "    if teclas.pressed(KeyCode::ArrowLeft) || teclas.pressed(KeyCode::KeyA) {\n",
        "        direcao.x -= 1.0;\n",
        "    }\n",
        "    if teclas.pressed(KeyCode::ArrowRight) || teclas.pressed(KeyCode::KeyD) {\n",
        "        direcao.x += 1.0;\n",
        "    }\n",
        "    if teclas.pressed(KeyCode::ArrowUp) || teclas.pressed(KeyCode::KeyW) {\n",
        "        direcao.y += 1.0;\n",
        "    }\n",
        "    if teclas.pressed(KeyCode::ArrowDown) || teclas.pressed(KeyCode::KeyS) {\n",
        "        direcao.y -= 1.0;\n",
        "    }\n\n",
        "    if direcao != Vec2::ZERO {\n",
        "        let passo = direcao.normalize() * tempo.delta_secs();\n",
        "        for (mut transform, jogador) in &mut query {\n",
        "            transform.translation.x += passo.x * jogador.velocidade;\n",
        "            transform.translation.y += passo.y * jogador.velocidade;\n",
        "        }\n",
        "    }\n",
        "}\n",
    )
}

/// Template completo do capstone `D10-P02 · Pong no terminal` (apenas std + #[cfg(test)]).
pub fn pong_terminal_starter_template() -> &'static str {
    concat!(
        "/// Estado lógico do Pong em terminal (`D10-P02`), separado da renderização.\n",
        "pub struct PongState {\n",
        "    pub largura: i32,\n",
        "    pub altura: i32,\n",
        "    pub raquete_esq_y: i32,\n",
        "    pub raquete_dir_y: i32,\n",
        "    pub bola_x: i32,\n",
        "    pub bola_y: i32,\n",
        "    pub vel_x: i32,\n",
        "    pub vel_y: i32,\n",
        "    pub placar_esq: u32,\n",
        "    pub placar_dir: u32,\n",
        "}\n\n",
        "impl PongState {\n",
        "    pub fn novo(largura: i32, altura: i32) -> Self {\n",
        "        let largura = largura.max(12);\n",
        "        let altura = altura.max(7);\n",
        "        let centro_y = altura / 2;\n",
        "        Self {\n",
        "            largura,\n",
        "            altura,\n",
        "            raquete_esq_y: (centro_y - 1).max(1),\n",
        "            raquete_dir_y: (centro_y - 1).max(1),\n",
        "            bola_x: largura / 2,\n",
        "            bola_y: centro_y,\n",
        "            vel_x: 1,\n",
        "            vel_y: 1,\n",
        "            placar_esq: 0,\n",
        "            placar_dir: 0,\n",
        "        }\n",
        "    }\n\n",
        "    pub fn mover_raquete_esq(&mut self, dy: i32) {\n",
        "        let min_y = 1;\n",
        "        let max_y = (self.altura - 4).max(min_y);\n",
        "        self.raquete_esq_y = (self.raquete_esq_y + dy).clamp(min_y, max_y);\n",
        "    }\n\n",
        "    fn reiniciar_bola(&mut self, dir_x: i32) {\n",
        "        self.bola_x = self.largura / 2;\n",
        "        self.bola_y = self.altura / 2;\n",
        "        self.vel_x = dir_x;\n",
        "    }\n\n",
        "    pub fn tick(&mut self) {\n",
        "        self.bola_x += self.vel_x;\n",
        "        self.bola_y += self.vel_y;\n\n",
        "        if self.bola_y <= 1 {\n",
        "            self.bola_y = 1;\n",
        "            self.vel_y = self.vel_y.abs();\n",
        "        } else if self.bola_y >= self.altura - 2 {\n",
        "            self.bola_y = self.altura - 2;\n",
        "            self.vel_y = -self.vel_y.abs();\n",
        "        }\n\n",
        "        if self.bola_x <= 2\n",
        "            && self.vel_x < 0\n",
        "            && (self.raquete_esq_y..=self.raquete_esq_y + 2).contains(&self.bola_y)\n",
        "        {\n",
        "            self.bola_x = 2;\n",
        "            self.vel_x = -self.vel_x;\n",
        "        } else if self.bola_x >= self.largura - 3\n",
        "            && self.vel_x > 0\n",
        "            && (self.raquete_dir_y..=self.raquete_dir_y + 2).contains(&self.bola_y)\n",
        "        {\n",
        "            self.bola_x = self.largura - 3;\n",
        "            self.vel_x = -self.vel_x;\n",
        "        }\n\n",
        "        if self.bola_x <= 0 {\n",
        "            self.placar_dir += 1;\n",
        "            self.reiniciar_bola(1);\n",
        "        } else if self.bola_x >= self.largura - 1 {\n",
        "            self.placar_esq += 1;\n",
        "            self.reiniciar_bola(-1);\n",
        "        }\n",
        "    }\n\n",
        "    pub fn renderizar_ascii(&self) -> String {\n",
        "        let mut out = format!(\"Placar {} x {}\\n\", self.placar_esq, self.placar_dir);\n",
        "        for y in 0..self.altura {\n",
        "            for x in 0..self.largura {\n",
        "                let ch = if y == 0 || y == self.altura - 1 {\n",
        "                    '#'\n",
        "                } else if x == self.bola_x && y == self.bola_y {\n",
        "                    'O'\n",
        "                } else if x == 1\n",
        "                    && (self.raquete_esq_y..=self.raquete_esq_y + 2).contains(&y)\n",
        "                {\n",
        "                    '|'\n",
        "                } else if x == self.largura - 2\n",
        "                    && (self.raquete_dir_y..=self.raquete_dir_y + 2).contains(&y)\n",
        "                {\n",
        "                    '|'\n",
        "                } else {\n",
        "                    ' '\n",
        "                };\n",
        "                out.push(ch);\n",
        "            }\n",
        "            out.push('\\n');\n",
        "        }\n",
        "        out\n",
        "    }\n",
        "}\n\n",
        "fn main() {\n",
        "    let mut jogo = PongState::novo(24, 9);\n",
        "    for _ in 0..5 {\n",
        "        jogo.tick();\n",
        "    }\n",
        "    println!(\"{}\", jogo.renderizar_ascii());\n",
        "}\n\n",
        "#[cfg(test)]\n",
        "mod tests {\n",
        "    use super::*;\n\n",
        "    #[test]\n",
        "    fn pong_rebate_raquete_e_marca_ponto() {\n",
        "        let mut jogo = PongState::novo(20, 9);\n",
        "        jogo.mover_raquete_esq(-10);\n",
        "        assert_eq!(jogo.raquete_esq_y, 1);\n",
        "        jogo.bola_x = 3;\n",
        "        jogo.bola_y = 2;\n",
        "        jogo.vel_x = -1;\n",
        "        jogo.tick();\n",
        "        assert!(jogo.vel_x > 0, \"bola deve rebater na raquete esquerda\");\n\n",
        "        jogo.bola_x = 1;\n",
        "        jogo.raquete_esq_y = 5;\n",
        "        jogo.vel_x = -1;\n",
        "        jogo.tick();\n",
        "        assert_eq!(jogo.placar_dir, 1);\n",
        "        assert!(jogo.renderizar_ascii().contains('O'));\n",
        "    }\n",
        "}\n",
    )
}

/// Template completo do capstone `D11-P02 · Snake no terminal` (apenas std + #[cfg(test)]).
pub fn snake_terminal_starter_template() -> &'static str {
    concat!(
        "use std::collections::VecDeque;\n\n",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n",
        "pub enum Direcao {\n",
        "    Cima,\n",
        "    Baixo,\n",
        "    Esquerda,\n",
        "    Direita,\n",
        "}\n\n",
        "pub struct SnakeGame {\n",
        "    pub largura: i32,\n",
        "    pub altura: i32,\n",
        "    pub corpo: VecDeque<(i32, i32)>,\n",
        "    pub direcao: Direcao,\n",
        "    pub comida: (i32, i32),\n",
        "    pub seed: u64,\n",
        "    pub game_over: bool,\n",
        "    pub pontos: u32,\n",
        "}\n\n",
        "impl SnakeGame {\n",
        "    pub fn novo(largura: i32, altura: i32, seed: u64) -> Self {\n",
        "        let largura = largura.max(6);\n",
        "        let altura = altura.max(6);\n",
        "        let cx = largura / 2;\n",
        "        let cy = altura / 2;\n",
        "        let mut corpo = VecDeque::new();\n",
        "        corpo.push_back((cx, cy));\n",
        "        corpo.push_back((cx - 1, cy));\n",
        "        corpo.push_back((cx - 2, cy));\n",
        "        let mut jogo = Self {\n",
        "            largura,\n",
        "            altura,\n",
        "            corpo,\n",
        "            direcao: Direcao::Direita,\n",
        "            comida: (cx + 1, cy),\n",
        "            seed,\n",
        "            game_over: false,\n",
        "            pontos: 0,\n",
        "        };\n",
        "        jogo.sortear_comida();\n",
        "        jogo\n",
        "    }\n\n",
        "    pub fn mudar_direcao(&mut self, nova: Direcao) {\n",
        "        let oposta = matches!(\n",
        "            (self.direcao, nova),\n",
        "            (Direcao::Cima, Direcao::Baixo)\n",
        "                | (Direcao::Baixo, Direcao::Cima)\n",
        "                | (Direcao::Esquerda, Direcao::Direita)\n",
        "                | (Direcao::Direita, Direcao::Esquerda)\n",
        "        );\n",
        "        if !oposta {\n",
        "            self.direcao = nova;\n",
        "        }\n",
        "    }\n\n",
        "    fn sortear_comida(&mut self) {\n",
        "        self.seed = self\n",
        "            .seed\n",
        "            .wrapping_mul(6364136223846793005)\n",
        "            .wrapping_add(1442695040888963407);\n",
        "        let total = (self.largura * self.altura) as usize;\n",
        "        let inicio = (self.seed as usize) % total.max(1);\n",
        "        for offset in 0..total {\n",
        "            let idx = (inicio + offset) % total;\n",
        "            let x = (idx as i32) % self.largura;\n",
        "            let y = (idx as i32) / self.largura;\n",
        "            if !self.corpo.contains(&(x, y)) {\n",
        "                self.comida = (x, y);\n",
        "                return;\n",
        "            }\n",
        "        }\n",
        "    }\n\n",
        "    pub fn passo(&mut self) {\n",
        "        if self.game_over {\n",
        "            return;\n",
        "        }\n",
        "        let Some(&(hx, hy)) = self.corpo.front() else {\n",
        "            self.game_over = true;\n",
        "            return;\n",
        "        };\n",
        "        let nova_cabeca = match self.direcao {\n",
        "            Direcao::Cima => (hx, hy - 1),\n",
        "            Direcao::Baixo => (hx, hy + 1),\n",
        "            Direcao::Esquerda => (hx - 1, hy),\n",
        "            Direcao::Direita => (hx + 1, hy),\n",
        "        };\n",
        "        if nova_cabeca.0 < 0\n",
        "            || nova_cabeca.0 >= self.largura\n",
        "            || nova_cabeca.1 < 0\n",
        "            || nova_cabeca.1 >= self.altura\n",
        "        {\n",
        "            self.game_over = true;\n",
        "            return;\n",
        "        }\n",
        "        let comeu = nova_cabeca == self.comida;\n",
        "        if !comeu {\n",
        "            self.corpo.pop_back();\n",
        "        }\n",
        "        if self.corpo.contains(&nova_cabeca) {\n",
        "            self.game_over = true;\n",
        "            return;\n",
        "        }\n",
        "        self.corpo.push_front(nova_cabeca);\n",
        "        if comeu {\n",
        "            self.pontos += 1;\n",
        "            self.sortear_comida();\n",
        "        }\n",
        "    }\n",
        "}\n\n",
        "fn main() {\n",
        "    let mut snake = SnakeGame::novo(12, 10, 42);\n",
        "    snake.passo();\n",
        "    println!(\"Pontos: {} | Cabeça: {:?}\", snake.pontos, snake.corpo.front());\n",
        "}\n\n",
        "#[cfg(test)]\n",
        "mod tests {\n",
        "    use super::*;\n\n",
        "    #[test]\n",
        "    fn snake_bloqueia_reversao_come_e_bate_na_parede() {\n",
        "        let mut s = SnakeGame::novo(10, 10, 7);\n",
        "        s.mudar_direcao(Direcao::Esquerda);\n",
        "        assert_eq!(s.direcao, Direcao::Direita);\n",
        "        let cabeca = *s.corpo.front().unwrap();\n",
        "        s.comida = (cabeca.0 + 1, cabeca.1);\n",
        "        let len_antes = s.corpo.len();\n",
        "        s.passo();\n",
        "        assert_eq!(s.pontos, 1);\n",
        "        assert_eq!(s.corpo.len(), len_antes + 1);\n\n",
        "        for _ in 0..20 {\n",
        "            s.passo();\n",
        "        }\n",
        "        assert!(s.game_over);\n",
        "    }\n",
        "}\n",
    )
}

/// Template completo do capstone `D12-P02 · Mini editor de texto com undo/redo`.
pub fn mini_editor_starter_template() -> &'static str {
    concat!(
        "/// Mini editor de texto com múltiplas linhas e histórico undo/redo (`D12-P02`).\n",
        "pub struct MiniEditor {\n",
        "    pub linhas: Vec<String>,\n",
        "    pub cursor_l: usize,\n",
        "    pub cursor_c: usize,\n",
        "    pub undo_stack: Vec<Vec<String>>,\n",
        "    pub redo_stack: Vec<Vec<String>>,\n",
        "}\n\n",
        "impl MiniEditor {\n",
        "    pub fn novo(texto: &str) -> Self {\n",
        "        let mut linhas: Vec<String> = texto.lines().map(ToString::to_string).collect();\n",
        "        if linhas.is_empty() {\n",
        "            linhas.push(String::new());\n",
        "        }\n",
        "        Self {\n",
        "            linhas,\n",
        "            cursor_l: 0,\n",
        "            cursor_c: 0,\n",
        "            undo_stack: Vec::new(),\n",
        "            redo_stack: Vec::new(),\n",
        "        }\n",
        "    }\n\n",
        "    fn byte_idx(linha: &str, col_char: usize) -> usize {\n",
        "        linha\n",
        "            .char_indices()\n",
        "            .nth(col_char)\n",
        "            .map(|(i, _)| i)\n",
        "            .unwrap_or(linha.len())\n",
        "    }\n\n",
        "    fn snapshot(&mut self) {\n",
        "        self.undo_stack.push(self.linhas.clone());\n",
        "        self.redo_stack.clear();\n",
        "    }\n\n",
        "    pub fn inserir_char(&mut self, ch: char) {\n",
        "        self.snapshot();\n",
        "        let idx = Self::byte_idx(&self.linhas[self.cursor_l], self.cursor_c);\n",
        "        self.linhas[self.cursor_l].insert(idx, ch);\n",
        "        self.cursor_c += 1;\n",
        "    }\n\n",
        "    pub fn quebrar_linha(&mut self) {\n",
        "        self.snapshot();\n",
        "        let idx = Self::byte_idx(&self.linhas[self.cursor_l], self.cursor_c);\n",
        "        let resto = self.linhas[self.cursor_l].split_off(idx);\n",
        "        self.cursor_l += 1;\n",
        "        self.linhas.insert(self.cursor_l, resto);\n",
        "        self.cursor_c = 0;\n",
        "    }\n\n",
        "    pub fn apagar_char(&mut self) {\n",
        "        if self.cursor_c > 0 {\n",
        "            self.snapshot();\n",
        "            let idx = Self::byte_idx(&self.linhas[self.cursor_l], self.cursor_c - 1);\n",
        "            self.linhas[self.cursor_l].remove(idx);\n",
        "            self.cursor_c -= 1;\n",
        "        } else if self.cursor_l > 0 {\n",
        "            self.snapshot();\n",
        "            let atual = self.linhas.remove(self.cursor_l);\n",
        "            self.cursor_l -= 1;\n",
        "            self.cursor_c = self.linhas[self.cursor_l].chars().count();\n",
        "            self.linhas[self.cursor_l].push_str(&atual);\n",
        "        }\n",
        "    }\n\n",
        "    pub fn desfazer(&mut self) -> bool {\n",
        "        let Some(prev) = self.undo_stack.pop() else {\n",
        "            return false;\n",
        "        };\n",
        "        self.redo_stack.push(self.linhas.clone());\n",
        "        self.linhas = prev;\n",
        "        self.cursor_l = self.cursor_l.min(self.linhas.len().saturating_sub(1));\n",
        "        self.cursor_c = self.cursor_c.min(self.linhas[self.cursor_l].chars().count());\n",
        "        true\n",
        "    }\n\n",
        "    pub fn refazer(&mut self) -> bool {\n",
        "        let Some(next) = self.redo_stack.pop() else {\n",
        "            return false;\n",
        "        };\n",
        "        self.undo_stack.push(self.linhas.clone());\n",
        "        self.linhas = next;\n",
        "        self.cursor_l = self.cursor_l.min(self.linhas.len().saturating_sub(1));\n",
        "        self.cursor_c = self.cursor_c.min(self.linhas[self.cursor_l].chars().count());\n",
        "        true\n",
        "    }\n\n",
        "    pub fn conteudo(&self) -> String {\n",
        "        self.linhas.join(\"\\n\")\n",
        "    }\n",
        "}\n\n",
        "fn main() {\n",
        "    let mut ed = MiniEditor::novo(\"Olá\");\n",
        "    ed.quebrar_linha();\n",
        "    ed.inserir_char('!');\n",
        "    println!(\"{}\", ed.conteudo());\n",
        "}\n\n",
        "#[cfg(test)]\n",
        "mod tests {\n",
        "    use super::*;\n\n",
        "    #[test]\n",
        "    fn mini_editor_edita_desfaz_e_refaz() {\n",
        "        let mut ed = MiniEditor::novo(\"abc\");\n",
        "        ed.cursor_c = 1;\n",
        "        ed.quebrar_linha();\n",
        "        assert_eq!(ed.conteudo(), \"a\\nbc\");\n",
        "        ed.inserir_char('X');\n",
        "        assert_eq!(ed.conteudo(), \"a\\nXbc\");\n",
        "        ed.apagar_char();\n",
        "        assert_eq!(ed.conteudo(), \"a\\nbc\");\n",
        "        assert!(ed.desfazer());\n",
        "        assert!(ed.desfazer());\n",
        "        assert_eq!(ed.conteudo(), \"a\\nbc\");\n",
        "        assert!(ed.refazer());\n",
        "        assert_eq!(ed.conteudo(), \"a\\nXbc\");\n",
        "    }\n",
        "}\n",
    )
}

/// Template completo do Major Capstone 1 `PAINT-01..12 · Mini Paint 2D`.
pub fn mini_paint_starter_template() -> &'static str {
    concat!(
        "/// Canvas 2D lógico desacoplado de redimensionamento da janela (`PAINT-01..12`).\n",
        "pub struct Canvas2D {\n",
        "    pub largura: usize,\n",
        "    pub altura: usize,\n",
        "    pub pixels: Vec<[u8; 3]>,\n",
        "    pub undo_stack: Vec<Vec<[u8; 3]>>,\n",
        "}\n\n",
        "impl Canvas2D {\n",
        "    pub fn novo(largura: usize, altura: usize) -> Self {\n",
        "        let largura = largura.max(1);\n",
        "        let altura = altura.max(1);\n",
        "        Self {\n",
        "            largura,\n",
        "            altura,\n",
        "            pixels: vec![[255, 255, 255]; largura * altura],\n",
        "            undo_stack: Vec::new(),\n",
        "        }\n",
        "    }\n\n",
        "    pub fn salvar_undo(&mut self) {\n",
        "        self.undo_stack.push(self.pixels.clone());\n",
        "    }\n\n",
        "    pub fn desfazer(&mut self) -> bool {\n",
        "        let Some(prev) = self.undo_stack.pop() else {\n",
        "            return false;\n",
        "        };\n",
        "        self.pixels = prev;\n",
        "        true\n",
        "    }\n\n",
        "    pub fn pintar_pixel(&mut self, x: usize, y: usize, cor: [u8; 3]) {\n",
        "        if x < self.largura && y < self.altura {\n",
        "            self.pixels[y * self.largura + x] = cor;\n",
        "        }\n",
        "    }\n\n",
        "    pub fn ler_pixel(&self, x: usize, y: usize) -> Option<[u8; 3]> {\n",
        "        (x < self.largura && y < self.altura).then(|| self.pixels[y * self.largura + x])\n",
        "    }\n\n",
        "    pub fn tracar_linha(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, cor: [u8; 3]) {\n",
        "        let dx = (x1 - x0).abs();\n",
        "        let sx = if x0 < x1 { 1 } else { -1 };\n",
        "        let dy = -(y1 - y0).abs();\n",
        "        let sy = if y0 < y1 { 1 } else { -1 };\n",
        "        let mut err = dx + dy;\n",
        "        loop {\n",
        "            if x0 >= 0 && y0 >= 0 {\n",
        "                self.pintar_pixel(x0 as usize, y0 as usize, cor);\n",
        "            }\n",
        "            if x0 == x1 && y0 == y1 {\n",
        "                break;\n",
        "            }\n",
        "            let e2 = 2 * err;\n",
        "            if e2 >= dy {\n",
        "                err += dy;\n",
        "                x0 += sx;\n",
        "            }\n",
        "            if e2 <= dx {\n",
        "                err += dx;\n",
        "                y0 += sy;\n",
        "            }\n",
        "        }\n",
        "    }\n\n",
        "    pub fn mapear_janela_para_canvas(\n",
        "        &self,\n",
        "        win_x: f32,\n",
        "        win_y: f32,\n",
        "        win_w: f32,\n",
        "        win_h: f32,\n",
        "    ) -> Option<(usize, usize)> {\n",
        "        if win_w <= 0.0 || win_h <= 0.0 || win_x < 0.0 || win_y < 0.0 || win_x >= win_w || win_y >= win_h {\n",
        "            return None;\n",
        "        }\n",
        "        let cx = ((win_x / win_w) * self.largura as f32).floor() as usize;\n",
        "        let cy = ((win_y / win_h) * self.altura as f32).floor() as usize;\n",
        "        Some((cx.min(self.largura - 1), cy.min(self.altura - 1)))\n",
        "    }\n\n",
        "    pub fn exportar_ppm(&self) -> String {\n",
        "        let mut out = format!(\"P3\\n{} {}\\n255\\n\", self.largura, self.altura);\n",
        "        for [r, g, b] in &self.pixels {\n",
        "            out.push_str(&format!(\"{r} {g} {b}\\n\"));\n",
        "        }\n",
        "        out\n",
        "    }\n",
        "}\n\n",
        "fn main() {\n",
        "    let mut canvas = Canvas2D::novo(16, 16);\n",
        "    canvas.salvar_undo();\n",
        "    canvas.tracar_linha(0, 0, 15, 15, [220, 40, 40]);\n",
        "    println!(\"Exportado PPM: {} bytes\", canvas.exportar_ppm().len());\n",
        "}\n\n",
        "#[cfg(test)]\n",
        "mod tests {\n",
        "    use super::*;\n\n",
        "    #[test]\n",
        "    fn mini_paint_preserva_canvas_no_resize_e_exporta_ppm() {\n",
        "        let mut canvas = Canvas2D::novo(32, 24);\n",
        "        canvas.salvar_undo();\n",
        "        canvas.tracar_linha(0, 0, 5, 5, [10, 20, 30]);\n",
        "        assert_eq!(canvas.ler_pixel(3, 3), Some([10, 20, 30]));\n",
        "        assert_eq!(canvas.mapear_janela_para_canvas(160.0, 120.0, 320.0, 240.0), Some((16, 12)));\n",
        "        assert_eq!(canvas.mapear_janela_para_canvas(320.0, 240.0, 640.0, 480.0), Some((16, 12)));\n",
        "        assert_eq!((canvas.largura, canvas.altura), (32, 24));\n",
        "        assert!(canvas.exportar_ppm().starts_with(\"P3\\n32 24\\n255\\n\"));\n",
        "        assert!(canvas.desfazer());\n",
        "        assert_eq!(canvas.ler_pixel(3, 3), Some([255, 255, 255]));\n",
        "    }\n",
        "}\n",
    )
}

/// Template completo do Ultimate Capstone `CAVE-01..18 · Mundo voxel 24x24 chunks (Bevy 0.20)`.
pub fn cavegame_voxel_starter_template() -> &'static str {
    concat!(
        "use std::collections::BTreeMap;\n\n",
        "pub const WORLD_CHUNKS_X: i32 = 24;\n",
        "pub const WORLD_CHUNKS_Z: i32 = 24;\n",
        "pub const CHUNK_SIZE: i32 = 16;\n\n",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n",
        "pub enum Voxel {\n",
        "    Ar,\n",
        "    Grama,\n",
        "    Terra,\n",
        "    Pedra,\n",
        "}\n\n",
        "pub fn global_para_chunk_e_local(x: i32, z: i32) -> ((i32, i32), (usize, usize)) {\n",
        "    let cx = x.div_euclid(CHUNK_SIZE);\n",
        "    let cz = z.div_euclid(CHUNK_SIZE);\n",
        "    let lx = x.rem_euclid(CHUNK_SIZE) as usize;\n",
        "    let lz = z.rem_euclid(CHUNK_SIZE) as usize;\n",
        "    ((cx, cz), (lx, lz))\n",
        "}\n\n",
        "pub fn chunk_dentro_do_mundo_24x24(cx: i32, cz: i32) -> bool {\n",
        "    (0..WORLD_CHUNKS_X).contains(&cx) && (0..WORLD_CHUNKS_Z).contains(&cz)\n",
        "}\n\n",
        "pub fn altura_terreno_seed(seed: u64, x: i32, z: i32) -> i32 {\n",
        "    let h = seed\n",
        "        .wrapping_add((x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))\n",
        "        .wrapping_add((z as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));\n",
        "    4 + ((h >> 12) % 8) as i32\n",
        "}\n\n",
        "pub struct VoxelWorld {\n",
        "    pub seed: u64,\n",
        "    pub blocos: BTreeMap<(i32, i32, i32), Voxel>,\n",
        "}\n\n",
        "impl VoxelWorld {\n",
        "    pub fn novo_com_seed(seed: u64) -> Self {\n",
        "        Self {\n",
        "            seed,\n",
        "            blocos: BTreeMap::new(),\n",
        "        }\n",
        "    }\n\n",
        "    pub fn get(&self, x: i32, y: i32, z: i32) -> Voxel {\n",
        "        self.blocos.get(&(x, y, z)).copied().unwrap_or(Voxel::Ar)\n",
        "    }\n\n",
        "    pub fn set(&mut self, x: i32, y: i32, z: i32, v: Voxel) -> bool {\n",
        "        let ((cx, cz), _) = global_para_chunk_e_local(x, z);\n",
        "        if !chunk_dentro_do_mundo_24x24(cx, cz) || y < 0 {\n",
        "            return false;\n",
        "        }\n",
        "        if v == Voxel::Ar {\n",
        "            self.blocos.remove(&(x, y, z));\n",
        "        } else {\n",
        "            self.blocos.insert((x, y, z), v);\n",
        "        }\n",
        "        true\n",
        "    }\n\n",
        "    pub fn faces_visiveis(&self, x: i32, y: i32, z: i32) -> usize {\n",
        "        if self.get(x, y, z) == Voxel::Ar {\n",
        "            return 0;\n",
        "        }\n",
        "        [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)]\n",
        "            .into_iter()\n",
        "            .filter(|(dx, dy, dz)| self.get(x + dx, y + dy, z + dz) == Voxel::Ar)\n",
        "            .count()\n",
        "    }\n\n",
        "    pub fn serializar(&self) -> String {\n",
        "        let mut out = format!(\"seed={}\\n\", self.seed);\n",
        "        for ((x, y, z), v) in &self.blocos {\n",
        "            let tag = match v {\n",
        "                Voxel::Ar => \"ar\",\n",
        "                Voxel::Grama => \"grama\",\n",
        "                Voxel::Terra => \"terra\",\n",
        "                Voxel::Pedra => \"pedra\",\n",
        "            };\n",
        "            out.push_str(&format!(\"{x},{y},{z}={tag}\\n\"));\n",
        "        }\n",
        "        out\n",
        "    }\n\n",
        "    pub fn desserializar(s: &str) -> Self {\n",
        "        let mut lines = s.lines();\n",
        "        let seed = lines\n",
        "            .next()\n",
        "            .and_then(|l| l.strip_prefix(\"seed=\"))\n",
        "            .and_then(|v| v.parse().ok())\n",
        "            .unwrap_or(0);\n",
        "        let mut mundo = Self::novo_com_seed(seed);\n",
        "        for line in lines {\n",
        "            let Some((coords, tag)) = line.split_once('=') else {\n",
        "                continue;\n",
        "            };\n",
        "            let parts: Vec<i32> = coords.split(',').filter_map(|p| p.parse().ok()).collect();\n",
        "            if parts.len() != 3 {\n",
        "                continue;\n",
        "            }\n",
        "            let v = match tag {\n",
        "                \"grama\" => Voxel::Grama,\n",
        "                \"terra\" => Voxel::Terra,\n",
        "                \"pedra\" => Voxel::Pedra,\n",
        "                _ => Voxel::Ar,\n",
        "            };\n",
        "            mundo.set(parts[0], parts[1], parts[2], v);\n",
        "        }\n",
        "        mundo\n",
        "    }\n",
        "}\n\n",
        "fn main() {\n",
        "    let mut mundo = VoxelWorld::novo_com_seed(2026);\n",
        "    let h = altura_terreno_seed(mundo.seed, 10, 10);\n",
        "    mundo.set(10, h, 10, Voxel::Grama);\n",
        "    println!(\"CaveGame seed {} | faces visíveis: {}\", mundo.seed, mundo.faces_visiveis(10, h, 10));\n",
        "}\n\n",
        "#[cfg(test)]\n",
        "mod tests {\n",
        "    use super::*;\n\n",
        "    #[test]\n",
        "    fn cavegame_chunks_24x24_culling_e_persistencia() {\n",
        "        assert_eq!(global_para_chunk_e_local(-1, -17), ((-1, -2), (15, 15)));\n",
        "        assert!(chunk_dentro_do_mundo_24x24(0, 23));\n",
        "        assert!(!chunk_dentro_do_mundo_24x24(24, 0));\n",
        "        assert_eq!(altura_terreno_seed(99, 5, 8), altura_terreno_seed(99, 5, 8));\n\n",
        "        let mut mundo = VoxelWorld::novo_com_seed(99);\n",
        "        assert!(mundo.set(4, 2, 4, Voxel::Grama));\n",
        "        assert!(!mundo.set(-1, 2, 4, Voxel::Pedra));\n",
        "        assert_eq!(mundo.faces_visiveis(4, 2, 4), 6);\n",
        "        assert!(mundo.set(5, 2, 4, Voxel::Terra));\n",
        "        assert_eq!(mundo.faces_visiveis(4, 2, 4), 5);\n\n",
        "        let salvo = mundo.serializar();\n",
        "        let restaurado = VoxelWorld::desserializar(&salvo);\n",
        "        assert_eq!(restaurado.get(4, 2, 4), Voxel::Grama);\n",
        "        assert_eq!(restaurado.get(5, 2, 4), Voxel::Terra);\n",
        "    }\n",
        "}\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glossario_tem_explicacao_e_documentacao_oficial() {
        let hint = lookup("println").unwrap();
        assert!(hint.summary.contains("quebra de linha"));
        assert_eq!(hint.url, "https://doc.rust-lang.org/std/macro.println.html");
        assert!(lookup("simbolo_desconhecido").is_none());
    }

    #[test]
    fn resolve_variavel_local_biblioteca_extra_e_docs_pt() {
        let code = "use macroquad::prelude::*;\nfn atualizar() {\n    let mut velocidade = 5;\n}\n";
        let var_doc = resolve_symbol_doc("velocidade", code).unwrap();
        assert!(var_doc.summary.contains("mutável"));
        assert!(
            var_doc
                .url_pt
                .as_deref()
                .unwrap()
                .contains("rust-book-pt-br")
        );

        let fn_doc = resolve_symbol_doc("atualizar", code).unwrap();
        assert!(fn_doc.summary.contains("Função `atualizar`"));

        let mq_doc = resolve_symbol_doc("draw_circle", code).unwrap();
        assert!(mq_doc.summary.contains("macroquad"));
        assert!(mq_doc.url_en.contains("docs.rs/macroquad"));
    }

    #[test]
    fn insere_use_e_dependencia_sem_duplicar() {
        let src = "fn main() {\n    println!(\"oi\");\n}\n";
        let updated = insert_use_line_in_rust_code(src, "use macroquad::prelude::*;");
        assert!(updated.starts_with("use macroquad::prelude::*;"));
        assert_eq!(
            insert_use_line_in_rust_code(&updated, "use macroquad::prelude::*;"),
            updated
        );

        let toml = "[package]\nname = \"jogo\"\nversion = \"0.1.0\"\n\n[dependencies]\n";
        let toml2 = ensure_dependency_in_cargo_toml(toml, "macroquad", "0.4");
        assert!(toml2.contains("macroquad = \"0.4\""));
        assert_eq!(
            ensure_dependency_in_cargo_toml(&toml2, "macroquad", "0.4"),
            toml2
        );
    }

    #[test]
    fn bevy_no_menu_glossary_e_perfil_dev_rapido() {
        assert!(EXTRA_LIBRARIES.iter().any(|l| l.name == "bevy"));
        for sym in [
            "Commands",
            "Query",
            "Component",
            "DefaultPlugins",
            "Transform",
        ] {
            let doc = lookup(sym).expect("símbolo Bevy esperado no glossário");
            assert!(doc.url.contains("docs.rs/bevy"));
        }
        let toml = "[package]\nname = \"jogobevy\"\nversion = \"0.1.0\"\n";
        let with_bevy = ensure_dependency_in_cargo_toml(toml, "bevy", "0.20");
        let tuned = ensure_bevy_fast_dev_profile_in_cargo_toml(&with_bevy);
        assert!(tuned.contains("[profile.dev]"));
        assert!(tuned.contains("opt-level = 1"));
        assert!(tuned.contains("[profile.dev.package.\"*\"]"));
        assert_eq!(ensure_bevy_fast_dev_profile_in_cargo_toml(&tuned), tuned);
        assert!(bevy_starter_template().contains("DefaultPlugins"));
    }

    #[test]
    fn templates_capstone_v3_nao_vazios_e_completos() {
        for (tpl, marker) in [
            (pong_terminal_starter_template(), "pub struct PongState"),
            (snake_terminal_starter_template(), "pub struct SnakeGame"),
            (mini_editor_starter_template(), "pub struct MiniEditor"),
            (mini_paint_starter_template(), "pub struct Canvas2D"),
            (cavegame_voxel_starter_template(), "pub struct VoxelWorld"),
        ] {
            assert!(
                tpl.contains("#[cfg(test)]"),
                "template sem testes: {marker}"
            );
            assert!(tpl.contains(marker), "marcador ausente: {marker}");
            assert!(tpl.contains("fn main()"), "template sem main: {marker}");
        }
    }

    #[test]
    fn catalogo_f9_tem_metadados_v3_completos_para_todas_bibliotecas() {
        assert_eq!(EXTRA_LIBRARIES.len(), 7);
        for lib in EXTRA_LIBRARIES {
            let meta = lib.v3_metadata();
            assert!(!meta.system_deps_linux.trim().is_empty(), "{}", lib.name);
            assert!(!meta.associated_v3_lessons.is_empty(), "{}", lib.name);
            assert!(!meta.common_pitfalls.trim().is_empty(), "{}", lib.name);
            assert!(!meta.tested_environments.trim().is_empty(), "{}", lib.name);
            assert!(!meta.review_status.trim().is_empty(), "{}", lib.name);
        }
    }

    #[test]
    fn doc_hint_e_bibliotecas_suportam_ingles() {
        let libs = extra_libraries(true);
        assert_eq!(libs.len(), 7);
        for lib in libs {
            let meta = lib.v3_metadata_for(true);
            assert!(!meta.system_deps_linux.trim().is_empty(), "{}", lib.name);
            assert!(!meta.common_pitfalls.trim().is_empty(), "{}", lib.name);
            assert!(!meta.review_status.trim().is_empty(), "{}", lib.name);
        }

        let hint = lookup_for("println", true).expect("println hint in EN");
        assert!(!hint.summary.trim().is_empty());
        assert!(hint.url.contains("rust-lang.org"));

        let resolved = resolve_symbol_doc_for("speed", "fn main() { let mut speed = 10; }", true)
            .expect("resolved local mutable binding");
        assert!(resolved.title.contains("mutable variable"));
        assert!(resolved.summary.contains("Local mutable variable"));
        assert!(!resolved.best_url_for(true).is_empty());

        let bevy = bevy_starter_template_for(true);
        assert!(bevy.contains("fn main()"));
        assert!(bevy.contains("App::new()"));
    }
}
