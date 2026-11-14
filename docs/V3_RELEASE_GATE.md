# V3 Release Gate — Checklist de Qualidade, Acessibilidade e Plataformas

## 1. Portão Automatizado Local Obrigatório

Todos os comandos abaixo fazem parte do portão V3 e devem estar verdes após a última edição:

- [x] `cargo fmt --all -- --check` — formatação sem divergências.
- [x] `cargo clippy --all-targets -- -D warnings` — zero warnings em todo o workspace.
- [x] `cargo test --all-targets` — testes unitários, snapshots UI (`tests/ui_snapshot.rs`), integração Cargo (`tests/cargo_run.rs`, `tests/exercise_flow.rs` incluindo `D09-P01` interativo e os 5 capstones `/pong`, `/snake`, `/editor`, `/paint`, `/cavegame`), `tests/project_flow.rs`, `tests/fx_finish.rs` e linter V3 das 213 entradas (`curriculum::v3_blueprint::tests`).
- [x] `cargo build --release` — binário otimizado compilado.
- [x] `python3 tests/tui_phase4.py target/release/rustlab` — fluxo E2E real em PTY de exercícios (consentimento, falha, erro de compilação, dica `F1`, sucesso e persistência).
- [x] `python3 tests/tui_phase6.py target/release/rustlab` — fluxo E2E real em PTY da mini-IDE (projeto, arquivo novo, salvar, stdin/EOF, processo real).

## 2. Portão da Arquitetura V3 (Modo Aprender + Prática + SRS + Capstones)

- [x] Painel Superior de Explicação alinhado à esquerda, rolável e separado visualmente do Painel Inferior de Quiz / Prática.
- [x] Estados exatos do mascote **Ferris** à direita quando a largura permite:
  - Neutro: `c(o.o)c`
  - Feliz: `c(ovo)c`
  - Encorajador / triste sem culpar o aluno: `c(o,x,o)c`
  - Confuso / incerto: `c(o,0)c`
- [x] Prática real com o editor existente da mini-IDE e compilação/execução Cargo (`CargoRun`, `CargoRunWithInput` via `stdin`/EOF e `CargoTest`) sem perda de código não salvo ao trocar foco (`Tab`/`Shift+Tab`).
- [x] Bônus de independência `100 + 100 XP` sem dicas, Dica 4 rotulada como `[Modo Estudo/Revisão]`, rótulos compactos de status (`PracticeOutcome::status_badge`), explicador offline de erros pt-BR e trava read-only na IA do Modo Aprender.
- [x] Trilha V3 completa (213 lições `Published` em 6 estágios, tecla `v`, gating por pré-requisitos + pré-visualização `Alt+V`) e revisão espaçada (`SrsCard` em `progresso.toml`, atalho `r`).
- [x] Catálogo `F9` com metadados pedagógicos e dependências Linux estendidas (`V3LibraryMetadata`), mais notas finais em [V3_RELEASE_NOTES.md](file:///home/zephyr/Downloads/rust teacher/docs/V3_RELEASE_NOTES.md).
- [x] Layout testado em larguras estreitas (`50x12`, `60x24`) e padrão (`90x30`).

## 3. Verificações Manuais e Itens Explicitamente Não Verificados Automaticamente

| Item | Tipo de verificação | Status atual |
| --- | --- | --- |
| Renderização visual em múltiplos emuladores de terminal e leitores de tela | Manual (humana) | Pendente revisão humana de contraste/acessibilidade |
| Áudio suave em dispositivo físico PulseAudio/PipeWire | Manual (humana) | Fallback mudo verificado automaticamente; hardware físico depende do host |
| Cross-compile Windows (`x86_64-pc-windows-gnu`) e abertura de janelas GUI (`eframe`/`minifb`/`macroquad`/`bevy`) | Ambiente Windows / Display X11/Wayland | **Não verificado em sandbox Linux headless sem MinGW** |
