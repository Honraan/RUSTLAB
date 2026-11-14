//! Glossário offline em português para símbolos Rust, bibliotecas extras
//! visuais/de jogos e variáveis/funções definidas no código do aluno.

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
        match self.name {
            "macroquad" => V3LibraryMetadata {
                system_deps_linux: "libx11-dev, libxi-dev, libgl1-mesa-dev, libasound2-dev",
                associated_v3_lessons: &["D07-P02", "D10-P02", "PAINT-01"],
                common_pitfalls: "Esquecer `next_frame().await` no loop congela a janela; usar `#[macroquad::main]` em vez de main síncrona.",
                tested_environments: "Linux X11/Wayland · Windows MinGW (compilação guiada)",
                review_status: "Auditada V3 · GUI depende de display gráfico ativo",
            },
            "bevy" => V3LibraryMetadata {
                system_deps_linux: "libasound2-dev, libudev-dev, libwayland-dev, libxkbcommon-dev, vulkan/mesa",
                associated_v3_lessons: &["CAVE-01", "CAVE-08", "CAVE-18"],
                common_pitfalls: "Compilação dev sem `[profile.dev.package.\"*\"] opt-level = 1` fica lenta; queries mutáveis concorrentes sem `Without<T>` geram erro de borrow ECS.",
                tested_environments: "Linux Vulkan/Wayland/X11 · Modo CaveGame núcleo voxel testado headless",
                review_status: "Auditada V3 (0.20) · Janela 3D depende de display/GPU",
            },
            "minifb" => V3LibraryMetadata {
                system_deps_linux: "libx11-dev, libxkbcommon-dev (X11/Wayland)",
                associated_v3_lessons: &["PAINT-04", "PAINT-07"],
                common_pitfalls: "Passar buffer com tamanho diferente de `largura * altura` falha em runtime; atualizar janela sempre com `is_open()`.",
                tested_environments: "Linux desktop · Windows",
                review_status: "Auditada V3 · GUI depende de display",
            },
            "eframe" => V3LibraryMetadata {
                system_deps_linux: "libxcb, libgtk-3-dev, OpenGL/glow",
                associated_v3_lessons: &["PAINT-01", "PAINT-12", "D12-P02"],
                common_pitfalls: "Guardar estado transitório em variáveis locais do `update()` zera tudo a cada frame; mutar apenas campos da struct do App.",
                tested_environments: "Linux X11/Wayland · Windows",
                review_status: "Auditada V3 · Interface imediata depende de display",
            },
            "tachyonfx" => V3LibraryMetadata {
                system_deps_linux: "Nenhuma dependência nativa extra (pure Rust terminal / ratatui)",
                associated_v3_lessons: &["F05-P01", "D10-P02"],
                common_pitfalls: "Chamar efeito terminado sem reiniciar `EffectTimer`; aplicar coordenadas fora da `area` do buffer.",
                tested_environments: "Linux TTY/PTY · Windows Terminal · Headless CI",
                review_status: "Auditada e testada em headless (fx_finish.rs)",
            },
            "rand" => V3LibraryMetadata {
                system_deps_linux: "Nenhuma dependência de sistema (pure Rust)",
                associated_v3_lessons: &["M08-P01", "D08-P02", "CAVE-02"],
                common_pitfalls: "Intervalo invertido `gen_range(10..5)` entra em panic; para testes determinísticos prefira sementes fixas (`SeedableRng`).",
                tested_environments: "Multiplataforma completa + Headless CI",
                review_status: "Auditada e testada (Pure Rust)",
            },
            _ => V3LibraryMetadata {
                system_deps_linux: "libasound2-dev (ALSA) ou pipewire-alsa em Linux",
                associated_v3_lessons: &["D07-P02", "D11-P02"],
                common_pitfalls: "Dropar `_stream` retornada por `OutputStream::try_default()` encerra o áudio imediatamente mesmo com `Sink` vivo.",
                tested_environments: "Linux ALSA/PipeWire + fallback silencioso verificado",
                review_status: "Auditada · Fallback mudo testado automaticamente",
            },
        }
    }
}

pub fn lookup(symbol: &str) -> Option<DocHint> {
    let entry = match symbol {
        "println" => DocHint {
            summary: "Imprime texto formatado e adiciona uma quebra de linha.",
            url: "https://doc.rust-lang.org/std/macro.println.html",
        },
        "print" => DocHint {
            summary: "Imprime texto formatado sem adicionar quebra de linha.",
            url: "https://doc.rust-lang.org/std/macro.print.html",
        },
        "format" => DocHint {
            summary: "Monta uma String usando a sintaxe de formatação do Rust.",
            url: "https://doc.rust-lang.org/std/macro.format.html",
        },
        "String" => DocHint {
            summary: "Texto UTF-8 que pode crescer e ser alterado.",
            url: "https://doc.rust-lang.org/std/string/struct.String.html",
        },
        "Vec" => DocHint {
            summary: "Lista redimensionável de valores do mesmo tipo.",
            url: "https://doc.rust-lang.org/std/vec/struct.Vec.html",
        },
        "Option" => DocHint {
            summary: "Representa um valor que pode existir (Some) ou faltar (None).",
            url: "https://doc.rust-lang.org/std/option/enum.Option.html",
        },
        "Result" => DocHint {
            summary: "Representa sucesso (Ok) ou erro (Err) numa operação.",
            url: "https://doc.rust-lang.org/std/result/enum.Result.html",
        },
        "Some" => DocHint {
            summary: "Variante de Option que contém um valor.",
            url: "https://doc.rust-lang.org/std/option/enum.Option.html#variant.Some",
        },
        "None" => DocHint {
            summary: "Variante de Option que representa ausência de valor.",
            url: "https://doc.rust-lang.org/std/option/enum.Option.html#variant.None",
        },
        "Ok" => DocHint {
            summary: "Variante de Result que contém um resultado bem-sucedido.",
            url: "https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok",
        },
        "Err" => DocHint {
            summary: "Variante de Result que contém um erro.",
            url: "https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err",
        },
        "mut" => DocHint {
            summary: "Permite alterar uma variável ou referência após criá-la.",
            url: "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html",
        },
        "fn" => DocHint {
            summary: "Declara uma função; parâmetros e retorno têm tipos explícitos.",
            url: "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html",
        },
        "let" => DocHint {
            summary: "Declara uma variável; por padrão, ela é imutável.",
            url: "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html",
        },
        "main" => DocHint {
            summary: "Função principal onde a execução do programa Rust começa.",
            url: "https://doc.rust-lang.org/book/ch01-02-hello-world.html",
        },
        "eprintln" => DocHint {
            summary: "Imprime mensagem formatada na saída de erro (stderr) com quebra de linha.",
            url: "https://doc.rust-lang.org/std/macro.eprintln.html",
        },
        "vec" => DocHint {
            summary: "Macro para criar um Vec preenchido rapidamente (ex.: vec![1, 2, 3]).",
            url: "https://doc.rust-lang.org/std/macro.vec.html",
        },
        "assert_eq" => DocHint {
            summary: "Verifica em testes se dois valores são iguais; falha com pânico se diferirem.",
            url: "https://doc.rust-lang.org/std/macro.assert_eq.html",
        },
        "assert" => DocHint {
            summary: "Verifica em testes se uma condição booleana é verdadeira.",
            url: "https://doc.rust-lang.org/std/macro.assert.html",
        },
        "todo" => DocHint {
            summary: "Marca um trecho ainda não implementado (entra em pânico se executado).",
            url: "https://doc.rust-lang.org/std/macro.todo.html",
        },
        "panic" => DocHint {
            summary: "Interrompe a thread atual exibindo uma mensagem de erro irrecuperável.",
            url: "https://doc.rust-lang.org/std/macro.panic.html",
        },
        "match" => DocHint {
            summary: "Compara um valor contra padrões exaustivos e escolhe o ramo correspondente.",
            url: "https://doc.rust-lang.org/book/ch06-02-match.html",
        },
        "if" => DocHint {
            summary: "Executa um ramo condicionalmente quando uma expressão bool é verdadeira.",
            url: "https://doc.rust-lang.org/book/ch03-05-control-flow.html",
        },
        "else" => DocHint {
            summary: "Ramo alternativo executado quando a condição de um if é falsa.",
            url: "https://doc.rust-lang.org/book/ch03-05-control-flow.html",
        },
        "loop" => DocHint {
            summary: "Laço infinito controlado explicitamente com break ou return.",
            url: "https://doc.rust-lang.org/book/ch03-05-control-flow.html#repeating-code-with-loop",
        },
        "while" => DocHint {
            summary: "Repete um bloco enquanto uma condição booleana continuar verdadeira.",
            url: "https://doc.rust-lang.org/book/ch03-05-control-flow.html#conditional-loops-with-while",
        },
        "for" | "in" => DocHint {
            summary: "Percorre cada elemento produzido por um iterador ou intervalo.",
            url: "https://doc.rust-lang.org/book/ch03-05-control-flow.html#looping-through-a-collection-with-for",
        },
        "break" => DocHint {
            summary: "Interrompe imediatamente um laço (loop, while ou for).",
            url: "https://doc.rust-lang.org/book/ch03-05-control-flow.html#repeating-code-with-loop",
        },
        "continue" => DocHint {
            summary: "Pula para a próxima iteração do laço atual.",
            url: "https://doc.rust-lang.org/book/ch03-05-control-flow.html#conditional-loops-with-while",
        },
        "struct" => DocHint {
            summary: "Define um tipo composto agrupando campos nomeados ou em tupla.",
            url: "https://doc.rust-lang.org/book/ch05-01-defining-structs.html",
        },
        "enum" => DocHint {
            summary: "Define um tipo que pode assumir uma entre várias variantes explícitas.",
            url: "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html",
        },
        "impl" => DocHint {
            summary: "Implementa métodos, funções associadas ou traits para um tipo.",
            url: "https://doc.rust-lang.org/book/ch05-03-method-syntax.html",
        },
        "trait" => DocHint {
            summary: "Define um contrato de métodos compartilhados entre diferentes tipos.",
            url: "https://doc.rust-lang.org/book/ch10-02-traits.html",
        },
        "use" => DocHint {
            summary: "Traz módulos, tipos ou funções para o escopo atual.",
            url: "https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html",
        },
        "mod" => DocHint {
            summary: "Declara ou carrega um módulo para organizar código e visibilidade.",
            url: "https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html",
        },
        "pub" => DocHint {
            summary: "Torna um item (função, campo, módulo) acessível fora do módulo atual.",
            url: "https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html",
        },
        "return" => DocHint {
            summary: "Encerra a função imediatamente devolvendo o valor informado.",
            url: "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#functions-with-return-values",
        },
        "const" | "static" => DocHint {
            summary: "Define uma constante com tipo explícito e valor fixo durante o programa.",
            url: "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#constants",
        },
        "async" | "await" => DocHint {
            summary: "Permite operações assíncronas (como esperar próximo frame no macroquad ou I/O).",
            url: "https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html",
        },
        "move" => DocHint {
            summary: "Transfere a posse das variáveis capturadas para dentro de uma closure ou thread.",
            url: "https://doc.rust-lang.org/book/ch13-01-closures.html",
        },
        "self" | "Self" => DocHint {
            summary: "Refere-se à instância atual (`self`) ou ao próprio tipo dentro de `impl`/`trait` (`Self`).",
            url: "https://doc.rust-lang.org/book/ch05-03-method-syntax.html",
        },
        "str" => DocHint {
            summary: "Fatia imutável de texto UTF-8 (&str), muito usada para emprestar texto.",
            url: "https://doc.rust-lang.org/std/primitive.str.html",
        },
        "usize" => DocHint {
            summary: "Inteiro sem sinal do tamanho de ponteiro, padrão para índices e comprimentos.",
            url: "https://doc.rust-lang.org/std/primitive.usize.html",
        },
        "i32" | "i64" => DocHint {
            summary: "Inteiro com sinal (aceita valores negativos, zero e positivos).",
            url: "https://doc.rust-lang.org/std/primitive.i32.html",
        },
        "u32" | "u64" | "u8" => DocHint {
            summary: "Inteiro sem sinal (apenas valores não negativos, útil para pixels e contagens).",
            url: "https://doc.rust-lang.org/std/primitive.u32.html",
        },
        "f32" | "f64" => DocHint {
            summary: "Número decimal de ponto flutuante (muito usado em coordenadas, física e jogos).",
            url: "https://doc.rust-lang.org/std/primitive.f64.html",
        },
        "bool" => DocHint {
            summary: "Tipo booleano com exatamente dois valores possíveis: true ou false.",
            url: "https://doc.rust-lang.org/std/primitive.bool.html",
        },
        "char" => DocHint {
            summary: "Caractere escalar Unicode de 4 bytes escrito entre aspas simples ('a').",
            url: "https://doc.rust-lang.org/std/primitive.char.html",
        },
        "HashMap" => DocHint {
            summary: "Mapa associativo chave → valor em std::collections::HashMap.",
            url: "https://doc.rust-lang.org/std/collections/struct.HashMap.html",
        },
        "Clone" | "clone" => DocHint {
            summary: "Trait e método para duplicar explicitamente um valor em memória.",
            url: "https://doc.rust-lang.org/std/clone/trait.Clone.html",
        },
        "Debug" | "derive" => DocHint {
            summary: "Implementa automaticamente traits como Debug/Clone via #[derive(...)].",
            url: "https://doc.rust-lang.org/std/fmt/trait.Debug.html",
        },
        "cfg" | "test" => DocHint {
            summary: "Atributo condicional de compilação (ex.: #[cfg(test)] e #[test] para testes).",
            url: "https://doc.rust-lang.org/book/ch11-01-writing-tests.html",
        },
        "Iterator" | "iter" => DocHint {
            summary: "Percorre uma sequência preguiçosamente; combina com map, filter e collect.",
            url: "https://doc.rust-lang.org/std/iter/trait.Iterator.html",
        },
        "collect" => DocHint {
            summary: "Consome um iterador transformando seus itens em Vec, String ou outra coleção.",
            url: "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect",
        },
        "map" => DocHint {
            summary: "Transforma cada elemento de um iterador (ou Option/Result) com uma closure.",
            url: "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map",
        },
        "filter" => DocHint {
            summary: "Mantém apenas os elementos de um iterador que satisfazem um predicado.",
            url: "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter",
        },
        "len" => DocHint {
            summary: "Retorna a quantidade de elementos (ou bytes no caso de str/String).",
            url: "https://doc.rust-lang.org/std/vec/struct.Vec.html#method.len",
        },
        "is_empty" => DocHint {
            summary: "Retorna true se a coleção ou texto não possuir nenhum elemento.",
            url: "https://doc.rust-lang.org/std/vec/struct.Vec.html#method.is_empty",
        },
        "push" => DocHint {
            summary: "Adiciona um novo elemento ao final de um Vec ou caractere em uma String.",
            url: "https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push",
        },
        "lines" => DocHint {
            summary: "Retorna um iterador sobre cada linha de texto.",
            url: "https://doc.rust-lang.org/std/primitive.str.html#method.lines",
        },
        "trim" => DocHint {
            summary: "Remove espaços e quebras de linha do início e do fim de um &str.",
            url: "https://doc.rust-lang.org/std/primitive.str.html#method.trim",
        },
        "parse" => DocHint {
            summary: "Converte texto em outro tipo (ex.: texto.parse::<i32>()) devolvendo Result.",
            url: "https://doc.rust-lang.org/std/primitive.str.html#method.parse",
        },
        "to_string" => DocHint {
            summary: "Converte o valor ou &str em uma String com posse.",
            url: "https://doc.rust-lang.org/std/string/trait.ToString.html",
        },
        "as_str" => DocHint {
            summary: "Empresta uma String como fatia &str sem alocar memória nova.",
            url: "https://doc.rust-lang.org/std/string/struct.String.html#method.as_str",
        },
        "unwrap" => DocHint {
            summary: "Extrai Some/Ok diretamente, mas entra em pânico se for None/Err.",
            url: "https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap",
        },
        "expect" => DocHint {
            summary: "Como unwrap(), mas permite informar uma mensagem explicando o motivo em caso de erro.",
            url: "https://doc.rust-lang.org/std/option/enum.Option.html#method.expect",
        },
        "unwrap_or" => DocHint {
            summary: "Extrai o valor de Option/Result ou retorna um valor padrão seguro.",
            url: "https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or",
        },
        // --- Bibliotecas extras: macroquad ---
        "macroquad" => DocHint {
            summary: "Mini engine de jogos 2D em Rust: janela gráfica, formas, sprite, teclado e loop async.",
            url: "https://docs.rs/macroquad/latest/macroquad/",
        },
        "clear_background" => DocHint {
            summary: "macroquad: preenche toda a tela com uma cor (chame no início de cada frame).",
            url: "https://docs.rs/macroquad/latest/macroquad/window/fn.clear_background.html",
        },
        "draw_circle" => DocHint {
            summary: "macroquad: desenha um círculo preenchido na posição (x, y) com raio r e cor.",
            url: "https://docs.rs/macroquad/latest/macroquad/shapes/fn.draw_circle.html",
        },
        "draw_rectangle" => DocHint {
            summary: "macroquad: desenha um retângulo preenchido em (x, y, largura, altura, cor).",
            url: "https://docs.rs/macroquad/latest/macroquad/shapes/fn.draw_rectangle.html",
        },
        "draw_line" => DocHint {
            summary: "macroquad: desenha um segmento de reta entre (x1, y1) e (x2, y2) com espessura e cor.",
            url: "https://docs.rs/macroquad/latest/macroquad/shapes/fn.draw_line.html",
        },
        "draw_text" => DocHint {
            summary: "macroquad: escreve texto na tela em (x, y) com tamanho de fonte e cor.",
            url: "https://docs.rs/macroquad/latest/macroquad/text/fn.draw_text.html",
        },
        "is_key_down" => DocHint {
            summary: "macroquad: retorna true enquanto a tecla especificada estiver pressionada.",
            url: "https://docs.rs/macroquad/latest/macroquad/input/fn.is_key_down.html",
        },
        "is_key_pressed" => DocHint {
            summary: "macroquad: retorna true apenas no frame em que a tecla acabou de ser apertada.",
            url: "https://docs.rs/macroquad/latest/macroquad/input/fn.is_key_pressed.html",
        },
        "mouse_position" => DocHint {
            summary: "macroquad: devolve a tupla (x, y) atual do ponteiro do mouse na janela.",
            url: "https://docs.rs/macroquad/latest/macroquad/input/fn.mouse_position.html",
        },
        "screen_width" => DocHint {
            summary: "macroquad: largura atual da janela do jogo em pixels (f32).",
            url: "https://docs.rs/macroquad/latest/macroquad/window/fn.screen_width.html",
        },
        "screen_height" => DocHint {
            summary: "macroquad: altura atual da janela do jogo em pixels (f32).",
            url: "https://docs.rs/macroquad/latest/macroquad/window/fn.screen_height.html",
        },
        "next_frame" => DocHint {
            summary: "macroquad: apresenta o quadro desenhado na janela e espera o próximo ciclo (.await).",
            url: "https://docs.rs/macroquad/latest/macroquad/window/fn.next_frame.html",
        },
        "get_frame_time" => DocHint {
            summary: "macroquad: tempo delta (em segundos) desde o último frame — útil para movimentação suave.",
            url: "https://docs.rs/macroquad/latest/macroquad/time/fn.get_frame_time.html",
        },
        "Color" => DocHint {
            summary: "macroquad: representa cor RGBA para formas, fundo e textos (ex.: RED, GOLD, Color::new).",
            url: "https://docs.rs/macroquad/latest/macroquad/color/struct.Color.html",
        },
        "KeyCode" => DocHint {
            summary: "macroquad: enumeração das teclas do teclado (KeyCode::Up, KeyCode::Space, etc.).",
            url: "https://docs.rs/macroquad/latest/macroquad/input/enum.KeyCode.html",
        },
        "Conf" => DocHint {
            summary: "macroquad: configurações iniciais da janela (título, largura, altura).",
            url: "https://docs.rs/macroquad/latest/macroquad/conf/struct.Conf.html",
        },
        "vec2" | "Vec2" => DocHint {
            summary: "macroquad/glam: vetor 2D (x, y) usado para posições e velocidades em jogos.",
            url: "https://docs.rs/macroquad/latest/macroquad/math/struct.Vec2.html",
        },
        // --- minifb ---
        "minifb" => DocHint {
            summary: "Biblioteca mínima de janela gráfica por buffer de pixels RGB 2D.",
            url: "https://docs.rs/minifb/latest/minifb/",
        },
        "Window" => DocHint {
            summary: "minifb: representa a janela gráfica aberta para desenho direto de pixels.",
            url: "https://docs.rs/minifb/latest/minifb/struct.Window.html",
        },
        "WindowOptions" => DocHint {
            summary: "minifb: opções da janela (redimensionamento, título, escala).",
            url: "https://docs.rs/minifb/latest/minifb/struct.WindowOptions.html",
        },
        "Key" => DocHint {
            summary: "minifb: teclas suportadas em `janela.is_key_down(Key::Escape)`.",
            url: "https://docs.rs/minifb/latest/minifb/enum.Key.html",
        },
        "update_with_buffer" => DocHint {
            summary: "minifb: envia um slice &[u32] de cores RGB para redesenhar toda a janela.",
            url: "https://docs.rs/minifb/latest/minifb/struct.Window.html#method.update_with_buffer",
        },
        "is_open" => DocHint {
            summary: "minifb: retorna true enquanto a janela ainda não foi fechada pelo usuário.",
            url: "https://docs.rs/minifb/latest/minifb/struct.Window.html#method.is_open",
        },
        // --- eframe / egui ---
        "eframe" => DocHint {
            summary: "Runner oficial desktop/web para interfaces gráficas imediatas com egui.",
            url: "https://docs.rs/eframe/latest/eframe/",
        },
        "egui" => DocHint {
            summary: "Biblioteca GUI imediata com botões, caixas de texto, sliders e layouts.",
            url: "https://docs.rs/egui/latest/egui/",
        },
        "run_native" => DocHint {
            summary: "eframe: abre a janela nativa executando a aplicação egui.",
            url: "https://docs.rs/eframe/latest/eframe/fn.run_native.html",
        },
        "CentralPanel" => DocHint {
            summary: "egui: área central responsiva onde ficam os controles principais da janela.",
            url: "https://docs.rs/egui/latest/egui/struct.CentralPanel.html",
        },
        "SidePanel" => DocHint {
            summary: "egui: painel lateral esquerdo/direito para menus e ferramentas.",
            url: "https://docs.rs/egui/latest/egui/struct.SidePanel.html",
        },
        "Ui" => DocHint {
            summary: "egui: construtor imediato de widgets (`ui.heading`, `ui.button`, `ui.label`).",
            url: "https://docs.rs/egui/latest/egui/struct.Ui.html",
        },
        "heading" | "button" | "label" | "Slider" => DocHint {
            summary: "egui: componentes visuais básicos (título, botão clicável, rótulo e controle deslizante).",
            url: "https://docs.rs/egui/latest/egui/struct.Ui.html#method.button",
        },
        // --- tachyonfx ---
        "tachyonfx" => DocHint {
            summary: "Biblioteca de efeitos visuais e animações por célula no terminal (fade, dissolve, sweep).",
            url: "https://docs.rs/tachyonfx/latest/tachyonfx/",
        },
        "fx" | "dissolve" | "fade_from_fg" | "sweep_in" => DocHint {
            summary: "tachyonfx: funções de fábrica para criar efeitos visuais animados composáveis.",
            url: "https://docs.rs/tachyonfx/latest/tachyonfx/fx/index.html",
        },
        "Effect" | "EffectTimer" | "Interpolation" => DocHint {
            summary: "tachyonfx: representa a duração, curva de suavização e execução de uma animação.",
            url: "https://docs.rs/tachyonfx/latest/tachyonfx/struct.Effect.html",
        },
        // --- rand ---
        "rand" => DocHint {
            summary: "Gerador de números aleatórios padrão da comunidade Rust.",
            url: "https://docs.rs/rand/latest/rand/",
        },
        "Rng" | "rng" | "random_range" | "random_bool" => DocHint {
            summary: "rand: trait e métodos para sortear intervalos (`rng.random_range(1..=10)`) e booleanos.",
            url: "https://docs.rs/rand/latest/rand/trait.Rng.html",
        },
        // --- rodio ---
        "rodio" => DocHint {
            summary: "Biblioteca de reprodução e síntese simples de áudio em Rust.",
            url: "https://docs.rs/rodio/latest/rodio/",
        },
        "OutputStream" | "Sink" | "SineWave" | "Source" => DocHint {
            summary: "rodio: tipos principais para abrir áudio, controlar fila de reprodução e gerar ondas sonoras.",
            url: "https://docs.rs/rodio/latest/rodio/struct.Sink.html",
        },
        // --- bevy (Engine 2D/3D ECS) ---
        "bevy" => DocHint {
            summary: "Engine de jogos 2D/3D em Rust baseada em arquitetura ECS (Entities, Components, Systems).",
            url: "https://docs.rs/bevy/latest/bevy/",
        },
        "App" | "add_plugins" | "add_systems" => DocHint {
            summary: "bevy: núcleo da aplicação (`App::new().add_plugins(DefaultPlugins).add_systems(Update, fn_sistema).run()`).",
            url: "https://docs.rs/bevy/latest/bevy/app/struct.App.html",
        },
        "DefaultPlugins" | "MinimalPlugins" | "Plugin" => DocHint {
            summary: "bevy: conjunto padrão de subsistemas (janela, renderizador, entrada, áudio e tempo).",
            url: "https://docs.rs/bevy/latest/bevy/struct.DefaultPlugins.html",
        },
        "Startup" | "Update" | "FixedUpdate" | "PostUpdate" => DocHint {
            summary: "bevy: agendas (Schedules) — Startup roda uma vez ao iniciar; Update roda todo frame.",
            url: "https://docs.rs/bevy/latest/bevy/app/struct.Startup.html",
        },
        "Commands" | "spawn" | "despawn" => DocHint {
            summary: "bevy ECS: enfileira comandos estruturais seguros no mundo (criar/remover entidades e componentes).",
            url: "https://docs.rs/bevy/latest/bevy/ecs/system/struct.Commands.html",
        },
        "Query" | "With" | "Without" | "Changed" | "Added" => DocHint {
            summary: "bevy ECS: consulta e itera rapidamente todas as entidades que possuem certos Componentes.",
            url: "https://docs.rs/bevy/latest/bevy/ecs/system/struct.Query.html",
        },
        "Res" | "ResMut" | "Resource" => DocHint {
            summary: "bevy ECS: acesso somente leitura (`Res<T>`) ou mutável (`ResMut<T>`) a recursos globais do jogo.",
            url: "https://docs.rs/bevy/latest/bevy/ecs/system/struct.Res.html",
        },
        "Local" => DocHint {
            summary: "bevy ECS: estado privado persistente exclusivo de um único sistema entre frames.",
            url: "https://docs.rs/bevy/latest/bevy/ecs/system/struct.Local.html",
        },
        "Component" | "Bundle" | "Entity" => DocHint {
            summary: "bevy ECS: `#[derive(Component)]` transforma structs/enums em dados anexáveis a entidades (`Entity`).",
            url: "https://docs.rs/bevy/latest/bevy/ecs/component/trait.Component.html",
        },
        "Event" | "EventReader" | "EventWriter" => DocHint {
            summary: "bevy ECS: barramento tipado de eventos entre sistemas sem acoplamento direto.",
            url: "https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html",
        },
        "Transform" | "GlobalTransform" | "Vec3" | "Quat" => DocHint {
            summary: "bevy: posição, rotação e escala 2D/3D de uma entidade (`Transform::from_xyz(x, y, z)`).",
            url: "https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html",
        },
        "Camera2d" | "Camera3d" => DocHint {
            summary: "bevy: cria uma câmera 2D ou 3D no mundo (`commands.spawn(Camera2d);`).",
            url: "https://docs.rs/bevy/latest/bevy/core_pipeline/core_2d/struct.Camera2d.html",
        },
        "Sprite" | "Text2d" | "Gizmos" => DocHint {
            summary: "bevy: componentes de renderização 2D (sprites retangulares, textos 2D e linhas/formas de debug).",
            url: "https://docs.rs/bevy/latest/bevy/sprite/struct.Sprite.html",
        },
        "Time" | "Timer" | "delta_secs" => DocHint {
            summary: "bevy: relógio do jogo (`tempo.delta_secs()`) para movimentação suave independente de FPS.",
            url: "https://docs.rs/bevy/latest/bevy/time/struct.Time.html",
        },
        "ButtonInput" | "just_pressed" | "pressed" => DocHint {
            summary: "bevy: estado de teclado/mouse via recurso (`teclas: Res<ButtonInput<KeyCode>>`).",
            url: "https://docs.rs/bevy/latest/bevy/input/struct.ButtonInput.html",
        },
        "AssetServer" | "Handle" | "Image" => DocHint {
            summary: "bevy: carregamento assíncrono de imagens, fontes e sons da pasta `assets/`.",
            url: "https://docs.rs/bevy/latest/bevy/asset/struct.AssetServer.html",
        },
        _ => return None,
    };
    Some(entry)
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
    lookup_pt_url(symbol).or_else(|| lookup(symbol).map(|h| h.url))
}

/// Resolve documentação para um símbolo sob o cursor/ponteiro: primeiro busca no
/// glossário Rust/bibliotecas extras; caso contrário, inspeciona declarações locais
/// (`let`, `let mut`, `fn`, `struct`, `enum`, `const`, parâmetros) no código aberto.
pub fn resolve_symbol_doc(symbol: &str, code: &str) -> Option<ResolvedDoc> {
    if let Some(hint) = lookup(symbol) {
        return Some(ResolvedDoc {
            title: symbol.to_string(),
            summary: hint.summary.to_string(),
            url_pt: lookup_pt_url(symbol).map(str::to_string),
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
                title: format!("{symbol} (variável mutável)"),
                summary: format!(
                    "Variável mutável local (`let mut {symbol}`) definida no seu código. Seu valor pode mudar durante a execução."
                ),
                url_pt: Some(
                    "https://rust-br.github.io/rust-book-pt-br/ch03-01-variables-and-mutability.html"
                        .to_string(),
                ),
                url_en:
                    "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html"
                        .to_string(),
            });
        }
        if contains_ident_after(trimmed, "let ", symbol) {
            return Some(ResolvedDoc {
                title: format!("{symbol} (variável)"),
                summary: format!(
                    "Variável imutável local (`let {symbol}`) definida no seu código. Em Rust, variáveis são imutáveis por padrão."
                ),
                url_pt: Some(
                    "https://rust-br.github.io/rust-book-pt-br/ch03-01-variables-and-mutability.html"
                        .to_string(),
                ),
                url_en:
                    "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html"
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
                summary: format!(
                    "Função `{symbol}` definida no seu projeto para organizar e reutilizar um bloco de lógica."
                ),
                url_pt: Some(
                    "https://rust-br.github.io/rust-book-pt-br/ch03-03-how-functions-work.html"
                        .to_string(),
                ),
                url_en: "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html"
                    .to_string(),
            });
        }
        if contains_ident_after(trimmed, "struct ", symbol)
            || contains_ident_after(trimmed, "pub struct ", symbol)
        {
            return Some(ResolvedDoc {
                title: format!("struct {symbol}"),
                summary: format!(
                    "Estrutura `{symbol}` criada no seu projeto para agrupar campos relacionados."
                ),
                url_pt: Some(
                    "https://rust-br.github.io/rust-book-pt-br/ch05-01-defining-structs.html"
                        .to_string(),
                ),
                url_en: "https://doc.rust-lang.org/book/ch05-01-defining-structs.html".to_string(),
            });
        }
        if contains_ident_after(trimmed, "enum ", symbol)
            || contains_ident_after(trimmed, "pub enum ", symbol)
        {
            return Some(ResolvedDoc {
                title: format!("enum {symbol}"),
                summary: format!(
                    "Enumeração `{symbol}` definida no seu projeto representando estados ou variantes possíveis."
                ),
                url_pt: Some(
                    "https://rust-br.github.io/rust-book-pt-br/ch06-01-defining-an-enum.html"
                        .to_string(),
                ),
                url_en: "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html".to_string(),
            });
        }
        if contains_ident_after(trimmed, "const ", symbol)
            || contains_ident_after(trimmed, "pub const ", symbol)
            || contains_ident_after(trimmed, "static ", symbol)
        {
            return Some(ResolvedDoc {
                title: format!("const {symbol}"),
                summary: format!("Constante `{symbol}` definida no seu código."),
                url_pt: Some(
                    "https://rust-br.github.io/rust-book-pt-br/ch03-01-variables-and-mutability.html"
                        .to_string(),
                ),
                url_en:
                    "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html"
                        .to_string(),
            });
        }
        if trimmed.contains("fn ") && trimmed.contains(&format!("{symbol}:")) {
            return Some(ResolvedDoc {
                title: format!("{symbol} (parâmetro)"),
                summary: format!("Parâmetro `{symbol}` recebido pela função no seu código."),
                url_pt: Some(
                    "https://rust-br.github.io/rust-book-pt-br/ch03-03-how-functions-work.html"
                        .to_string(),
                ),
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

/// Código inicial Bevy 0.20 (ECS 2D) comentado em português para estudantes.
pub fn bevy_starter_template() -> &'static str {
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
}
