# V4 Baseline Audit — Auditoria do Repositório RustLab (Checkout Real)

**Data da auditoria:** 25 de novembro de 2026  
**Escopo:** Fase V4.0 — levantamento factual do estado atual do repositório, compilação, testes, currículo, mini-IDE, catálogo `F9`, incompatibilidades Windows, persistência e licença antes de qualquer alteração de código para o **RustLab 4.0 — Windows Desktop Edition**.

> [!IMPORTANT]
> Todos os dados abaixo refletem exclusivamente arquivos lidos e comandos executados diretamente no checkout atual. Nenhuma compilação ou execução gráfica no Windows foi simulada ou presumida.

---

## 1. Metadados do Checkout e Ambiente Observado

### 1.1 Repositório e Manifesto
- **Diretório raiz:** `c:\Users\drago\Documents\GitHub\RUSTLAB`
- **Branch / Commit (`.git/HEAD` e `.git/logs/HEAD`):** `main` em `3ccbb11cbf1382cd94c2be8c2ce6091747cfa3f4` (`clone: from https://github.com/Honraan/RUSTLAB.git`)
- **Pacote (`Cargo.toml`):**
  - `name = "rustlab"`
  - `version = "0.3.2"`
  - `edition = "2024"`
  - `license = "GPL-3.0-only"` (confirmado em `LICENSE` — GNU General Public License v3.0, 674 linhas)
- **Dependências diretas (`Cargo.toml`):**
  - `ratatui = { version = "0.30", features = ["unstable-rendered-line-info"] }`
  - `tachyonfx = "0.25"`
  - `serde = { version = "1", features = ["derive"] }`
  - `toml = "1"`
  - `tokio = { version = "1", features = ["process", "io-util", "time", "rt", "sync", "macros"] }`
  - `libc = "0.2"`
  - `[dev-dependencies]`: vazio.
- **Volume atual do código:**
  - `src/`: 41 arquivos `.rs` (~42.272 linhas no total)
  - `src/content/`: 52 arquivos `.toml` (10 capítulos em `chapter/`, 30 lições em `lesson/`, 12 exercícios em `exercise/`)
  - `tests/`: 6 arquivos `.rs` (`cargo_run.rs`, `cli.rs`, `exercise_flow.rs`, `fx_finish.rs`, `project_flow.rs`, `ui_snapshot.rs`), 2 scripts Python PTY (`tui_phase4.py`, `tui_phase6.py`) e 28 arquivos de snapshot em `tests/snapshots/*.txt`

### 1.2 Ambiente Windows do Host de Auditoria
- **Sistema Operacional:** Microsoft Windows 10 Home, Versão `10.0.19045` (Build `19045`), arquitetura 64 bits (`x86_64`)
- **Ferramentas detectadas no host:**
  - `rustc 1.100.0 (0ef1bc386 2026-11-09)` e `cargo 1.100.0 (3acc79c0c 2026-11-05)` em `C:\Users\drago\.cargo\bin\`
  - `rustup`: toolchain ativa padrão `1.100.0-x86_64-pc-windows-gnullvm` (único target instalado: `x86_64-pc-windows-gnullvm`; `x86_64-pc-windows-msvc` **não** instalado)
  - `rust-analyzer.exe` em `C:\Users\drago\.cargo\bin\`
  - **Microsoft Edge WebView2 Runtime:** instalado, versão `158.0.4370.33` (confirmado em `HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`)
  - `curl.exe` (`C:\Windows\system32\curl.exe`) e `winget.exe`
- **Ferramentas ausentes no host:**
  - Linker `x86_64-w64-mingw32-clang` (necessário para a toolchain `gnullvm` ativa)
  - Visual Studio / MSVC C++ Build Tools (`cl.exe`, `link.exe`, `vswhere.exe`, Windows SDK)
  - `node`, `npm`, `pnpm`, `bun`
  - `makensis` (NSIS), `wix`
  - `git` (`git.exe` ausente no `PATH` e em `C:\Program Files\Git`)

---

## 2. Resultados Reais de `fmt`, `clippy` e `test`

Comandos executados diretamente no workspace atual em 25/11/2026:

| Comando | Código de saída | Resultado real observado | Causa / Diagnóstico |
|---|---:|---|---|
| `cargo fmt -- --check` | `0` | **PASSOU** | Formatação do repositório está conforme `rustfmt`. |
| `cargo clippy --all-targets` | `101` | **FALHOU** | `error: linker 'x86_64-w64-mingw32-clang' not found` ao compilar build scripts/proc-macros (`quote`, `proc-macro2`, `prettyplease`, `parking_lot_core`, `serde`, `serde_core`, `thiserror`, `rustversion`, `zmij`, `winapi`, `indoc`). |
| `cargo test` | `101` | **FALHOU** | Mesmo erro de linker `x86_64-w64-mingw32-clang` ausente antes da compilação das crates do `rustlab`. |

> [!WARNING]
> **Bloqueio duplo real no Windows:**
> 1. **Infraestrutura do host:** a toolchain `x86_64-pc-windows-gnullvm` não possui seu linker (`x86_64-w64-mingw32-clang`) no host, e a toolchain recomendada para Windows desktop/Tauri (`x86_64-pc-windows-msvc` + VS C++ Build Tools + Windows SDK) ainda não está instalada.
> 2. **Código-fonte atual:** mesmo com toolchain MSVC instalada, `rustlab` **ainda não compila no Windows** porque `src/storage.rs`, `src/execution.rs` e `src/projects.rs` (testes) importam e chamam APIs exclusivamente Unix (`std::os::unix::*` e `libc::killpg`).
> 3. **CI atual (`.github/workflows/ci.yml` e `.github/workflows/release.yml`):** executa apenas em `ubuntu-latest` / `ubuntu-22.04`, sem nenhum job `windows-latest`.

---

## 3. Inventário de Módulos, Fronteiras e Plano de Reuso

| Módulo (`src/`) | Linhas | Acoplamento observado | Classificação V4 | Notas de reuso / refatoração |
|---|---:|---|---|---|
| `src/lib.rs` | 30 | `pure-rust` | **Reuso direto** | Expõe módulos da biblioteca; base para extração em workspace. |
| `src/main.rs` | 85 | `ratatui`, `crossterm` | **TUI-only preservado** | Casca do terminal (`--doctor`, `--help`, `--version`, `IsTerminal`, panic hook). |
| `src/app.rs` | 1.743 | `ratatui`, `crossterm` | **Desacoplar UI / TUI adapter** | Orquestra `Screen`, `Config`, `Progress`, `ExecutionManager`, `DailyReviewState`, `OnboardingTourState`; precisa separar estado de negócio de `KeyEvent`/`Terminal`. |
| `src/event.rs` | 76 | `ratatui`, `crossterm` | **TUI-only preservado** | Loop de eventos `crossterm` em thread separada. |
| `src/theme.rs` | 176 | `ratatui` | **Desacoplar / espelhar tokens** | Define 4 paletas (`Cuia`, `Ferrugem`, `Mono`, `Oceano`); tokens semânticos devem ser mapeados para CSS/WebView2. |
| `src/animation.rs` | 171 | `ratatui`, `tachyonfx` | **TUI-only preservado** | Efeitos TUI + `MotionMode`; no desktop `MotionMode` será respeitado por CSS/Web Animations. |
| `src/audio.rs` | 277 | `pure-rust` (Linux-only em runtime) | **Adaptar para Windows / Desktop** | Usa `paplay`, `XDG_RUNTIME_DIR` e `/dev/snd`; reaproveitar fórmula `xp_pcm` (110 ms, 660 Hz) via Web Audio API no desktop e `#[cfg(unix)]` na TUI. |
| `src/config.rs` | 778 | `pure-rust` | **Adaptar caminhos & default pt-BR** | Usa `XDG_CONFIG_HOME`/`XDG_CACHE_HOME`/`HOME` e define `default_language() -> "en-US"`. |
| `src/storage.rs` | 331 | `unix/libc` (`OpenOptionsExt`) | **Adaptar para Windows** | `progresso.toml` v1, escrita atômica e backup `.corrupt-*` são excelentes; trocar `OpenOptionsExt::mode` e caminhos `XDG_DATA_HOME`/`HOME` com `#[cfg]`. |
| `src/i18n.rs` | 355 | `pure-rust` | **Reuso direto + expansão** | Centraliza strings `pt-BR`/`en-US` via `bi()`/`t_for()`. |
| `src/json.rs` | 515 | `pure-rust` | **Reuso direto (interno)** | Parser/serializador JSON mínimo usado por `lsp.rs` e `mentor.rs`; no backend Tauri pode conviver ou migrar gradualmente para `serde_json`. |
| `src/curriculum/mod.rs` | 812 | `pure-rust` | **Reuso direto** | Carrega e valida 10 capítulos, 30 lições e 12 exercícios TOML embutidos (`include_str!`). |
| `src/curriculum/v3_blueprint.rs` | 4.385 | `tokio` (não usado diretamente aqui; detectado por string) | **Reuso direto** | Fonte canônica das **213 entradas**, linter DAG, gating por estágio/pré-requisito, `build_v3_lesson_for`, `build_v3_practice_spec_for`. |
| `src/curriculum/en_tr.rs` | 2.966 | `pure-rust` | **Reuso direto** | Traduções `en-US` para currículo clássico e V3. |
| `src/learn.rs` | 3.619 | `ratatui`, `crossterm` | **Desacoplar `KeyEvent`** | Contém `FerrisMood` (`c(o.o)c`, `c(ovo)c`, `c(o,x,o)c`, `c(o,0)c`), `calculate_v3_practice_xp`, `PracticeSpec` (31 specs), `Progress`, SRS e explicador offline pt-BR; `KeyEvent` fica no adaptador TUI. |
| `src/exercises.rs` | 1.760 | `ratatui`, `crossterm` | **Desacoplar `KeyEvent`** | `ExerciseSession`, `ExerciseOutcome`, 8 desafios `bugfix-*`, harness temporário `rustlab_validation` e fluxo de dicas. |
| `src/editor.rs` | 898 | `pure-rust` | **Reuso na TUI / espelho no IDE** | Buffer `TextBuffer`, undo/redo, busca, seleção; no Desktop o frontend usa CodeMirror 6 sincronizado com arquivos/rascunhos no backend. |
| `src/highlight.rs` | 220 | `ratatui` | **TUI-only preservado** | Realce sintático para Ratatui; no Desktop o CodeMirror 6 assume o realce Rust. |
| `src/completion.rs` | 233 | `pure-rust` | **Reuso direto** | Autocompletar local offline de palavras-chave e símbolos. |
| `src/projects.rs` | 524 | `unix/libc` (em testes `symlink`) | **Adaptar para Windows** | `Project`, proteção anti-path-traversal, criação segura de arquivos, `PLANO.md`, `inspect_capabilities`; precisa tratar prefixo `\\?\`, junctions/reparse points e `%LOCALAPPDATA%`/`Documents`. |
| `src/execution.rs` | 969 | `unix/libc`, `tokio` | **Adaptar para Windows + ConPTY** | Enum fechado `ToolCommand`, sanitização de nomes/deps, streaming 256 KiB, `parse_diagnostics`. Precisa substituir `process_group(0)`/`killpg` por Windows Job Objects, adicionar `.exe` na busca, corrigir `parse_short`/`parse_arrow` para `C:\...` e adicionar ConPTY para interativos. |
| `src/lsp.rs` | 1.288 | `pure-rust` | **Adaptar URIs e `.exe` no Windows** | `WorkspaceIndex` offline + `infer_local_inlay_hints` + cliente `LspClient` JSON-RPC para `rust-analyzer`; `path_to_uri`/`uri_to_path` e `find_rust_analyzer` precisam suporte Windows (`file:///C:/...`, `.exe`, `USERPROFILE`). |
| `src/doctor.rs` | 77 | `pure-rust` | **Reuso + evolução Windows** | Diagnóstico CLI `--doctor`; base para o Toolchain Doctor V4 (adicionar `rustup`, `rustfmt`, `clippy`, MSVC `cl.exe`/`link.exe`, Windows SDK e WebView2). |
| `src/doc_hint.rs` | 2.228 | `ratatui`, `tachyonfx` | **Desacoplar pequenos trechos** | Glossário offline, catálogo `F9` (`EXTRA_LIBRARIES`, `V3LibraryMetadata`) e templates dos 5 capstones + Bevy. Contém apenas `ratatui` em exemplos/textos. |
| `src/create.rs` | 5.955 | `ratatui`, `crossterm`, `process` | **Desacoplar UI + substituir helpers Windows** | Máquina de estados do Modo Criar, comandos `/pong`, `/snake`, `/editor`, `/paint`, `/cavegame`, `/bevy`, F9, build menu. Contém `cmd /C start` (`L4770`) e `rundll32` (`L4933`) a remover no V4. |
| `src/mentor.rs` | 3.414 | `ratatui`, `crossterm` | **Desacoplar `KeyEvent`** | Política read-only (`is_learn_mode_context`, `sanitize_learn_mode_response`), `<dados_nao_confiaveis>`, `classify_code_authorship`, cliente Ollama e OpenAI-compatible. |
| `src/mentor/https.rs` | 94 | `pure-rust` (`curl`) | **Substituir no V4** | Usa `Command::new("curl")` com `--config -` e arquivo temporário `@path`; vulnerável a escaping de `\` no Windows e gravação de payload em disco. |
| `src/lab_tools.rs` | 5.268 | `ratatui`, `tokio`, `process` | **Desacoplar UI** | 10 ferramentas pedagógicas (`MemoryXRayReport`, `SocraticErrorLens`, `VisualDebuggerState`, `TroubleshootingQuest`, Playground export, `LifetimeMap`, etc.). |
| `src/ui/*.rs` (12 arquivos) | 6.727 | `ratatui`, `tachyonfx`, `crossterm` | **TUI-only preservado** | Telas TUI Classic (`browser`, `create`, `exercises`, `help`, `lesson`, `mentor`, `menu`, `mod`, `placeholder`, `progress`, `settings`, `welcome`). |

---

## 4. Auditoria Honesta das 213 Entradas do Currículo e Capstones

### 4.1 Integridade Estrutural Verificada em `src/curriculum/v3_blueprint.rs`
A contagem exata no array `V3_CURRICULUM_BLUEPRINT` confirma:

| Estágio | Composição | Teoria | Prática / Marco | Total | Status no código |
|---|---|---:|---:|---:|---|
| **O BÁSICO** (`V3Stage::Basico`) | `5T -> B-P01 -> 7T -> B-P02` | 12 | 2 | **14** | 14 `Published` |
| **O MEIO** (`V3Stage::Meio`) | `12 ciclos × (3T + 1P)` | 36 | 12 | **48** | 48 `Published` |
| **O DIFÍCIL** (`V3Stage::Dificil`) | `D-REV-001 + 12 ciclos × (3T + 2P)` | 36 | 25 | **61** | 61 `Published` |
| **O FIM** (`V3Stage::Fim`) | `10 unidades × (4T + 2P)` | 40 | 20 | **60** | 60 `Published` |
| **MINI PAINT** (`V3Stage::MiniPaint`) | `PAINT-01..PAINT-12` | 0 | 12 | **12** | 12 `Published` |
| **CAVEGAME (BEVY)** (`V3Stage::CaveGame`) | `CAVE-01..CAVE-18` | 0 | 18 | **18** | 18 `Published` |
| **TOTAL** | — | **124** | **89** | **213** | **213 `Published`, 0 `Drafted`, 0 `Tested`, 0 `Planned`** |

### 4.2 Distinção Real: Conteúdo Autoral TOML vs. Composição Gerada
Para evitar falsas premissas na migração V4, registramos exatamente como as 213 entradas são preenchidas hoje:
1. **Arquivos TOML autorais em `src/content/`:**
   - **10** capítulos (`etapa-1.toml` .. `etapa-10.toml`).
   - **30** lições completas em TOML; no blueprint, **34** entradas possuem `mapped_lesson_id = Some(...)` apontando para essas **30** lições distintas.
   - **12** exercícios TOML em `src/content/exercise/` (9 entradas do blueprint referenciam `mapped_exercise_id = Some(...)`); além disso, `src/exercises.rs` possui **8** micro-desafios `bugfix-*`.
2. **Composição dinâmica nas demais entradas (`build_v3_lesson_for`, `v3_blueprint.rs:3580`):**
   - Todas as 213 entradas geram uma `Lesson` completa e não-vazia combinando metadados próprios da entrada (`id`, `title`, `learning_goal`, `completion_evidence`, `prerequisites`), o callout `error_first_callout_for`, até 4 blocos herdados da lição TOML canônica via `canonical_lesson_for_v3_entry`, um bloco de código, um bloco `Demo` e um `Quiz` de 4 opções.
   - **Implicação para V4:** não há placeholders vazios (`planned = 0`), mas as 183 entradas além das 30 lições TOML compartilham blocos didáticos das 30 lições-base (ainda assim preservando IDs, objetivos, pré-requisitos e gating DAG próprios). A V4 preservará integralmente esses 213 IDs e só alterará conteúdo por revisão explícita.
3. **Práticas executáveis (`PracticeSpec`):**
   - `src/learn.rs` (`PRACTICE_SPECS`) contém **31** especificações executáveis: **1** `CargoRunWithInput` (`D09-P01`), **6** `CargoRun` e **24** `CargoTest`.
   - `v3_blueprint.rs` (`build_v3_practice_spec_for`, linhas `3204–3474`) adiciona **22** braços customizados (`M03-P01`, `M04-P01`, `D-REV-001`, grupos `D07`/`D08`/`D10`/`D11`/`D12`, `PAINT-*`, `CAVE-*`, `F01-P02..F06-P01`); alguns braços são compartilhados entre marcos adjacentes (ex.: `PAINT-01..03`, `PAINT-04..06`, `PAINT-07..09`, `PAINT-10..12`, `CAVE-01..04`, `CAVE-05..09`, `CAVE-10..18`).
4. **Estado real dos 5 Templates Capstone (`src/doc_hint.rs` & `src/create.rs:3241`):**
   - `/pong`, `/snake`, `/editor`, `/paint` e `/cavegame` possuem templates Rust com `#[cfg(test)]` testáveis em `cargo test`.
   - **Limitação honesta crítica:**
     - `mini_paint_starter_template` (`/paint`, `src/doc_hint.rs:1852`) implementa o núcleo `Canvas2D` em memória (traço Bresenham, undo, mapeamento janela→canvas, exportação PPM P3) e um `main()` de console, **sem abrir janela GUI `eframe`/`minifb` no starter**.
     - `cavegame_voxel_starter_template` (`/cavegame`, `src/doc_hint.rs:1963`) implementa o núcleo `VoxelWorld` `24×24` chunks (`16×16`), culling de faces e serialização por seed em `BTreeMap`, **sem cena/janela 3D Bevy no starter** (embora o comando `/cavegame` instale `bevy = "0.20"` no `Cargo.toml` e exista `bevy_starter_template_for` separado para 2D).
     - Nenhum dos capstones gráficos (`Mini Paint` GUI e `CaveGame` Bevy 3D) foi compilado ou renderizado em janela Windows real neste checkout.
5. **Divergência de idioma padrão (`src/config.rs:430-432`):**
   - `default_language()` retorna `"en-US"` e o teste `config::tests::idioma_padrao_e_ingles_na_primeira_abertura` (`src/config.rs:762`) exige esse comportamento.
   - A especificação V4 exige inicialização do **Windows Desktop em `pt-BR`**. A arquitetura V4 manterá `"en-US"` como default da camada TUI legada (para preservar regressões) e configurará `pt-BR` no onboarding/default da edição Windows Desktop.

---

## 5. Matriz de Capacidades Reais da Mini-IDE Atual

| Capacidade | TUI Atual (`src/create.rs`, `src/learn.rs`, `src/exercises.rs`) | Lacuna / Adaptação necessária no Windows Desktop V4 |
|---|---|---|
| Edição multilinha, seleção, undo/redo, busca | Implementado em `src/editor.rs` (`TextBuffer` UTF-8) | Adotar **CodeMirror 6** no frontend sincronizado com rascunhos/arquivos do backend; clipboard nativo do sistema funciona nativamente. |
| Árvore de arquivos, criar `.rs`, abrir/criar projeto | Implementado em `src/projects.rs` e `src/create.rs` | Adicionar seletores nativos Windows (`dialog`), suporte a `\\?\` e abas explícitas no frontend. |
| `cargo check` / `build` / `test` / `fmt` / `clippy` | `src/execution.rs` via `tokio::process::Command` | Trocar `process_group`/`killpg` por **Windows Job Objects**; resolver `cargo.exe`; corrigir `parse_diagnostics` para caminhos `C:\...`. |
| Execução interativa (`stdin`, jogos, editor TUI) | `RunInteractive` usa **pipes por linha** (não é PTY; sem raw input, setas ou ANSI) | Implementar **Windows ConPTY** + `xterm.js` no frontend para D09, Pong (`D10`), Snake (`D11`), Mini Editor (`D12`) e Modo Criar. |
| LSP (`rust-analyzer`) + fallback local | `src/lsp.rs` (`LspClient` + `WorkspaceIndex` + `infer_local_inlay_hints`) | Corrigir `find_rust_analyzer` (`.exe`, `USERPROFILE`) e `path_to_uri`/`uri_to_path` (`file:///C:/...`). |
| Execução de apps GUI separados | `spawn_external_window_binary` (`src/create.rs:4718`) | Suportar lançamento direto sem shell no Windows, acompanhamento de status/saída e erro claro se driver/GPU falhar. |
| Cancelamento e limites | Timeout por comando, `MAX_STREAM_BYTES = 256 KiB`, drenagem anti-deadlock | Preservar exatamente os mesmos limites; garantir encerramento da árvore via Job Object no Windows. |

---

## 6. Auditoria do Catálogo `F9` (`src/doc_hint.rs`)

| Crate | Versão em `F9` (`src/doc_hint.rs`) | Versão real verificada | Lições V3 associadas no código | Riscos e lacunas Windows |
|---|---|---|---|---|
| `macroquad` | `0.4` | — | `D07-P02`, `D10-P02`, `PAINT-01` | Requer MSVC/OpenGL/DirectX no Windows; não testado neste checkout. |
| `bevy` | `0.20` | `0.20.1` (crates.io, out/2026) | `CAVE-01`, `CAVE-08`, `CAVE-18` | Build inicial pesado; exige MSVC C++ Build Tools + Windows SDK + GPU compatível (DX12/Vulkan); `[profile.dev.package."*"] opt-level = 1` recomendado. |
| `minifb` | `0.27` | — | `PAINT-04`, `PAINT-07` | Janela Win32 nativa; requer compilação Windows real na V4.4. |
| `eframe` | `0.31` | — | `PAINT-01`, `PAINT-12`, `D12-P02` | Requer OpenGL/WGL (glow) ou wgpu no Windows; não testado neste checkout. |
| `tachyonfx` | `0.11` | `0.25` no `Cargo.toml` do host | `F05-P01`, `D10-P02` | **Divergência de versão:** `F9` lista `0.11` enquanto o próprio RustLab compila com `0.25`. Auditar e alinhar na V4.4 antes de gerar código aluno. |
| `rand` | `0.9` | — | `M08-P01`, `D08-P02`, `CAVE-02` | Pure Rust; API `0.9` (`rand::rng()`, `random_range`); compilação Windows pendente de teste. |
| `rodio` | `0.20` | — | `D07-P02`, `D11-P02` | WASAPI no Windows; verificar degradação silenciosa sem dispositivo de áudio. |

Além da divergência de `tachyonfx`, `V3LibraryMetadata` (`src/doc_hint.rs:219`) possui apenas `system_deps_linux`, sem campo estruturado para pré-requisitos Windows (`system_deps_windows` / requisitos GPU).

---

## 7. Inventário Completo de Incompatibilidades Windows e Riscos Técnicos

### 7.1 Bloqueadores de Compilação no Windows
1. `src/storage.rs:7` — `use std::os::unix::fs::OpenOptionsExt;` e `src/storage.rs:149` — `.mode(0o600)`.
2. `src/execution.rs:17` — `use std::os::unix::fs::PermissionsExt;` e `src/execution.rs:185` — `(metadata.permissions().mode() & 0o111) != 0`.
3. `src/execution.rs:440` — `use std::os::unix::process::CommandExt;` e `src/execution.rs:480` — `ac.as_std_mut().process_group(0)`.
4. `src/execution.rs:583` — `libc::killpg(pid as libc::pid_t, libc::SIGKILL)`.
5. `src/projects.rs:443,458` — `std::os::unix::fs::symlink` nos testes unitários.

### 7.2 Falhas de Runtime, Caminhos, Processos e Segurança no Windows
1. **Caminhos XDG/HOME hardcoded:**
   - `src/storage.rs:55-60` (`XDG_DATA_HOME`, `HOME/.local/share`)
   - `src/config.rs:390-404, 492-504` (`XDG_CACHE_HOME`, `XDG_CONFIG_HOME`, `HOME`)
   - `src/projects.rs:264, 277-287` (`expand_home` só olha `HOME`, `default_workspace_dir` olha `XDG_DATA_HOME`/`HOME`)
   - `src/app.rs:572,575` (caminhos `/tmp/rustlab-snapshot-*.toml` em snapshot mode)
2. **Descoberta de binários sem `.exe`:**
   - `src/execution.rs:153-179` (`cargo_candidates` procura `cargo` sem `.exe` e pastas `/usr/...`, `/opt/cargo/...`, sem `USERPROFILE`)
   - `src/lsp.rs:505-514` (`find_rust_analyzer` procura `rust-analyzer` sem `.exe` e sem `USERPROFILE`)
3. **Parser de diagnósticos quebra em `C:\...`:**
   - `src/execution.rs:741-778`: `parse_short` usa `line.splitn(4, ':')` e `parse_arrow` usa `rest.trim().split(':')`. Em caminhos Windows como `C:\Users\aluno\src\main.rs:12:5: error[E0308]: ...`, o primeiro `:` é da letra de unidade `C:`, quebrando `file`, `line` e `col`.
4. **LSP `file://` URIs inválidas no Windows:**
   - `src/lsp.rs:1108-1144`: `path_to_uri` não normaliza `\` para `/`, não remove prefixo verbatim `\\?\` e não antepõe `/` antes de `C:/...` (padrão esperado pelo `rust-analyzer`: `file:///C:/...`). `uri_to_path` devolve `/C:/...` no Windows.
5. **Canonicalização Windows e Reparse Points:**
   - `src/projects.rs:28, 166-175, 212-219`: `Path::canonicalize()` no Windows devolve caminhos UNC verbatim (`\\?\C:\...`), que podem falhar em ferramentas externas, quebrar `strip_prefix` se misturados a caminhos normais e confundir iniciantes na UI. Além disso, `collect_rs` (`src/projects.rs:234,243`) checa `is_symlink()`, sendo necessário garantir cobertura explícita para *junctions* e *reparse points* do NTFS.
   - `src/projects.rs:380`: teste `lista_fontes_sem_target_e_ordena` compara strings literais com `/` (`"src/lib.rs"`), o que falha no Windows com `\`.
6. **Target de release Windows inadequado para build nativo:**
   - `src/execution.rs:65-71` (`ToolCommand::BuildWindowsRelease`) e `src/create.rs:1273,1339` usam `x86_64-pc-windows-gnu` (cross-compile via MinGW a partir do Linux). No Windows Desktop V4, a compilação nativa do aluno usa a toolchain do host (`x86_64-pc-windows-msvc` por padrão).
7. **Helpers legados perigosos em `src/create.rs`:**
   - `src/create.rs:4770`: `Command::new("cmd").args(["/C", "start", program])` — passa argumentos por `cmd.exe` (risco de interpretação de metacaracteres `& | > < ^ %`).
   - `src/create.rs:4933`: `Command::new("rundll32").args(["url.dll,FileProtocolHandler", url])` — superfície frágil para abertura de URLs; no V4 Desktop deve ser substituído por opener oficial Tauri com validação estrita `https://`.
8. **Transporte HTTPS via `curl --config -` (`src/mentor/https.rs:32,39`):**
   - `data-binary = "@{path}"` escreve o payload JSON (contendo código/prompt) em `%TEMP%\rustlab-mentor-*.json`, e caminhos Windows com `\` dentro de aspas no arquivo de configuração do `curl` são tratados como escapes pelo parser do `curl`. No V4 Desktop o transporte HTTPS deve usar cliente HTTP Rust em memória (`reqwest` com TLS nativo/rustls) sem arquivo temporário.
9. **Áudio e PTY de testes:**
   - `src/audio.rs`: 100% específico de Linux (`paplay`, `XDG_RUNTIME_DIR`, `/dev/snd`).
   - `tests/tui_phase4.py` e `tests/tui_phase6.py`: importam `pty`, `fcntl`, `termios` (somente Unix).
   - `tests/cli.rs:20` e `tests/cargo_run.rs:7`: dependem de `HOME`.

---

## 8. Formatos Reais de Dados e Estratégia de Migração

- **Progresso (`src/storage.rs`):** arquivo TOML `progresso.toml` com `version = 1`, `read_lessons`, `completed_lessons`, `completed_exercises`, `hints_used`, `exercise_answers`, `xp_events`, `project_milestones`, `srs_cards` (`SrsCard`), `error_counts` e `onboarding_completed`. Escrita atômica via `.tmp-*` + `sync_all` + `rename` e preservação de arquivo corrompido em `.corrupt-*`; recusa gravação se `version > 1`.
- **Configuração (`src/config.rs`):** `config.toml` com `language`, `origin_language`, `ui` (`theme`, `animations`, `gamification`, `inlay_hints`), `sound` (`enabled`, `volume`), `ai` (`provider`, `ollama_url`, `ollama_model`, `remote_url`, `remote_model`, `remote_consent`), `execution` (`consent`, `shared_target_cache`, `target_cache_dir`) e `workspace` (`last_project`, `recent_projects`, `projects_dir`).
- **Projetos (`src/projects.rs`):** projetos Cargo padrão (`Cargo.toml`, `src/*.rs`, `tests/`, `examples/`, `benches/`, `build.rs`, `PLANO.md`). Nenhum formato proprietário bloqueia o estudante.
- **Estratégia V4:** manter compatibilidade total de `progresso.toml` (evolução aditiva `v1 -> v2` com `#[serde(default)]`), mover armazenamento padrão no Windows para `%LOCALAPPDATA%\RustLab\` e oferecer importação/exportação portátil explícita para usuários migrando da TUI Linux (detalhado em [`V4_DATA_MIGRATION.md`](./V4_DATA_MIGRATION.md)).

---

## 9. Auditoria de Assets Não-Código e Licença

- **Assets binários no repositório:** **zero** (nenhum `.png`, `.ico`, `.svg`, `.wav`, `.ogg`, `.ttf`, `.woff2`).
  - Mascote Ferris é representado exclusivamente em texto ASCII (`c(o.o)c`, `c(ovo)c`, `c(o,x,o)c`, `c(o,0)c`).
  - Som de XP é sintetizado em PCM puro (`xp_pcm` em `src/audio.rs`).
- **Licença (`LICENSE` & `Cargo.toml`):** `GPL-3.0-only`.
  - **Regra V4:** não alterar a licença sem autorização expressa dos mantenedores. Todo ícone, ilustração SVG do Ferris, fonte empacotada (se houver) e efeito sonoro adicionado na V4 deverá ser original do projeto ou possuir licença compatível com `GPL-3.0-only` e atribuição registrada em `Sobre / Licenças`.
