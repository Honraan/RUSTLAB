# Catálogo de Bibliotecas `F9` (`src/doc_hint.rs`)

Fonte em código: `EXTRA_LIBRARIES` em `src/doc_hint.rs` e menu modal `F9` / botão `[ ▼ Bibliotecas (F9) ]` em `src/create.rs` + `src/ui/create.rs`.

## Entradas Atuais

1. **macroquad (`0.4`)** — *Mini Engine de Jogos 2D*
   - Uso sugerido: jogos 2D simples, loop `next_frame().await`, formas e teclado.
   - `use` inserido: `use macroquad::prelude::*;`
   - Docs: `https://docs.rs/macroquad/latest/macroquad/`
   - Status: listado no `F9`, glossário offline ativo; build externo depende de cache/rede e libs de vídeo.

2. **bevy (`0.20`)** — *Engine de Jogos 2D/3D (ECS)*
   - Uso sugerido: **Ultimate Capstone CaveGame (`CAVE-01..CAVE-18`)**.
   - Template rápido (`T`): gera projeto com `DefaultPlugins`, sistema ECS exemplo e `[profile.dev]` otimizado para dependências em `Cargo.toml`.
   - Execução recomendada: janela nativa separada ou terminal paralelo via `F4`.
   - `use` inserido: `use bevy::prelude::*;`
   - Docs: `https://docs.rs/bevy/latest/bevy/`

3. **minifb (`0.27`)** — *Janela Gráfica & Pixels 2D*
   - Uso sugerido: introdução a framebuffers `Vec<u32>` e candidato ao `Mini Paint`.
   - `use` inserido: `use minifb::{Key, Window, WindowOptions};`
   - Docs: `https://docs.rs/minifb/latest/minifb/`

4. **eframe (`0.31`)** — *Interface Gráfica (GUI egui)*
   - Uso sugerido: apps desktop com painéis e candidato principal ao `Mini Paint`.
   - `use` inserido: `use eframe::egui;`
   - Docs: `https://docs.rs/eframe/latest/eframe/`

5. **tachyonfx (`0.11`)** — *Efeitos Visuais & Animações*
   - Uso sugerido: transições em terminal Ratatui (`fade_from_fg`, `dissolve`).
   - Observação: o próprio RustLab compila com `tachyonfx = "0.25"`. Ao criar lições no Modo Aprender para `F9`, validar versão fixada exata.
   - `use` inserido: `use tachyonfx::{Effect, EffectTimer, Interpolation, fx};`

6. **rand (`0.9`)** — *Sorteios & Aleatoriedade*
   - Uso sugerido: `M03-P01` (Adivinhe o número), `D11` (Snake com seed) e `CAVE-11` (terreno procedural com seed).
   - `use` inserido: `use rand::Rng;`

7. **rodio (`0.20`)** — *Áudio & Efeitos Sonoros*
   - Uso sugerido: áudio opcional em jogos, respeitando dispositivos ausentes.
   - `use` inserido: `use rodio::{OutputStream, Sink, source::SineWave};`

## Política de Extensão
Ao inserir uma dependência pelo menu `F9`, o RustLab atualiza o `Cargo.toml` apenas dentro do projeto atual após ação explícita do estudante, inserindo o `use` apropriado no topo do arquivo aberto sem executar comandos de rede ocultos antes do `cargo` autorizado.
