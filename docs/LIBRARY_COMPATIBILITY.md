# Matrix de Compatibilidade — Bibliotecas `F9`, Toolchain e Alvos Linux/Windows

Auditoria honesta baseada exclusivamente no ambiente verificado em execução.

## 1. Ambiente Local Observado

| Componente | Valor observado | Status |
| --- | --- | --- |
| Compilador `rustc` | `1.99.0` (2026-09-28) | Verificado localmente (Edição 2024 ativa) |
| Gerenciador `cargo` | `1.99.0` (2026-08-27) | Verificado localmente |
| Alvo nativo Linux | `cargo test --all-targets` / `cargo build --release` | Verificado e verde |
| Cross-compile Windows (`x86_64-pc-windows-gnu`) | `rustup` e `x86_64-w64-mingw32-gcc` ausentes no ambiente atual | **Não verificado aqui** (exige target + linker MinGW ou CI Windows dedicado; nunca declarar verde sem rodar) |
| Dependências do binário `rustlab` | `ratatui 0.30`, `tachyonfx 0.25`, `tokio 1`, `serde 1`, `toml 1`, `libc 0.2` | Compiladas e testadas no build principal |

## 2. Matriz das Bibliotecas Iniciais do Catálogo `F9` (`src/doc_hint.rs`)

> Importante: uma entrada listada no menu `F9` fornece metadados, descrição `pt-BR`, glossário offline, template e inserção no `Cargo.toml` sob consentimento. Compilação real no projeto do aluno depende de conexão inicial para download de crates (exceto modo offline com cache) e bibliotecas de sistema para GUI/áudio no SO alvo.

| Crate | Versão exibida em `F9` | Uso pedagógico planejado | Dependências / Riscos conhecidos | Status de compilação no projeto-aluno |
| --- | --- | --- | --- | --- |
| `macroquad` | `0.4` | Jogos 2D / arcade simples | Exige X11/Wayland/mesa-libGL no Linux; janela nativa separada | **Pendente de teste isolado de template V3.2+** |
| `bevy` | `0.20` | Capstone CaveGame (ECS 3D) | Compilação inicial pesada; exige ALSA/libudev/Vulkan/Wayland/X11 no Linux; configurado com `[profile.dev]` opt-level 1/3 no template | **Template e docs verificados; compilação de projeto Bevy externo não rodada neste ambiente offline** |
| `minifb` | `0.27` | Candidato leve para janela de pixels 2D (Mini Paint) | Exige X11/xkbcommon no Linux | **Pendente de teste em V3.7** |
| `eframe` | `0.31` | Candidato principal para Mini Paint GUI (`egui`) | Exige OpenGL/winit libs no Linux | **Pendente de teste em V3.7** |
| `tachyonfx` | `0.11` (no `rustlab` host: `0.25`) | Efeitos visuais em terminal Ratatui | Sem dependência gráfica nativa; atenção ao alinhamento de versão entre `0.11` (catálogo `F9`) e `0.25` (host) ao autorar lições | **Host `0.25` verificado; catálogo `0.11` requer validação na autoria V3.6** |
| `rand` | `0.9` | Sementes determinísticas e jogos (Snake, Adivinhe) | API `0.9` usa `rand::rng()` / `random_range()` (diferente de `0.8` `thread_rng`) | **API `0.9` documentada no `F9`; teste de template pendente** |
| `rodio` | `0.20` | Áudio em jogos e exercícios | Exige backend de áudio (ALSA/Pulse/PipeWire); deve degradar graciosamente sem hardware | **Pendente de teste isolado de template** |

## 3. Regra para Novas Bibliotecas (Seção I4)

Nenhuma aula futura pode adicionar crates fora da tabela acima silenciosamente. Toda nova dependência (como `clap`, `serde_json`, `sqlx`/`rusqlite`, `image`, `proptest`) deverá antes passar por auditoria de licença, manutenção, compilação isolada e atualização deste arquivo e de `F9_LIBRARY_CATALOG.md`.
