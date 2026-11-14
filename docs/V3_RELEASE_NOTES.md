# RustLab V3 — Notas de Lançamento (Certificação Final V3.0–V3.9)

Todas as fases da arquitetura **RustLab V3** estão concluídas e integradas ao aplicativo de terminal (`TUI`), mantendo o currículo todo em **português brasileiro (`pt-BR`)** e o compromisso de execução real via `cargo`.

---

## Principais Entregas

### 1. Trilha Completa de 213 Lições em 6 Estágios (`V3.0`–`V3.6`)
- **213 entradas publicadas (`Published`)** distribuídas exatamente nos 6 estágios da matriz canônica (`validate_v3_blueprint` + `build_v3_lesson` + `build_v3_practice_spec`):
  - **O BÁSICO (14):** 12 teorias + 2 práticas (`B-P01`, `B-P02`, ritmo `5T + P01 + 7T + P02`).
  - **O MEIO (48):** 12 ciclos no ritmo `3T + 1P`.
  - **O DIFÍCIL (61):** Revisão `D-REV-001` + 12 ciclos no ritmo `3T + 2P`, incluindo prática interativa por `stdin`/EOF (`D09-P01`) e os capstones de terminal (`/pong`, `/snake`, `/editor`).
  - **O FIM Core (60):** 10 unidades no ritmo `4T + 2P` cobrindo traits, iteradores, concorrência, I/O, CLI e arquitetura.
  - **MINI PAINT (`PAINT-01`..`PAINT-12` — 12 marcos):** Canvas lógico desacoplado de redimensionamento, traço contínuo, histórico e exportação PPM.
  - **CAVEGAME / BEVY (`CAVE-01`..`CAVE-18` — 18 marcos):** Coordenadas euclidianas de chunk, culling de faces, geração determinística por seed e integração `bevy = "0.20"`.
- A tela **Aprender** já inicia diretamente na **Trilha V3 completa (213 entradas em 6 estágios)**; pressione **`V`** (`v`/`V`) para alternar para a visão clássica (10 capítulos / 30 lições) e voltar, e **`Alt+V`** para habilitar/desabilitar a pré-visualização livre de estágios bloqueados por pré-requisitos.

### 2. Redesign do Modo Aprender, Mini-IDE Cargo e Ferris (`V3.1`–`V3.2`)
- **Separação vertical clara:** Painel Superior de Explicação alinhado à esquerda (com rolagem independente) e Painel Inferior de Quiz / Prática Cargo embutida (`Tab` alterna o foco sem descartar código não salvo).
- **4 estados canônicos do mascote Ferris à direita:** Neutro `c(o.o)c`, Feliz `c(ovo)c`, Encorajador `c(o,x,o)c` e Confuso `c(o,0)c`.
- **Execução real Cargo embutida (`F5`):** modos `CargoRun`, `CargoRunWithInput` (`stdin`/EOF) e `CargoTest`, com rótulos compactos de status (`[Aprovado]`, `[Erro de compilação]`, `[Tempo esgotado]`, etc.) e explicador offline pt-BR para diagnósticos do `rustc` sem penalizar XP.
- **Sistema de Dicas e Bônus de Independência:** 4 níveis progressivos (`F1`, `F2`/`F3` para navegar, Dica 4 marcada como `[Modo Estudo/Revisão]`) e recompensa **`100 + 100 XP`** ao concluir práticas sem dicas nem auxílio de IA.
- **Trava de IA Read-Only em Modo Aprender:** no Modo Aprender, o agente explica erros e conceitos sem aplicar edições diretas no editor.

### 3. Maestria e Revisão Espaçada Determinística (`V3.5`)
- Cartões `SrsCard` persistidos em `progresso.toml` (`version` retrocompatível via `serde(default)`).
- Relógio lógico monotônico sem pressão punitiva, atalho **`r`** no catálogo de curso e painel dedicado no **Progresso**.

### 4. Capstones (`/pong`, `/snake`, `/editor`, `/paint`, `/cavegame`) e Catálogo `F9` Auditado (`V3.7`–`V3.8`)
- Todos os 5 templates Capstone compilam com testes unitários reais verificados em integração Cargo (`tests/exercise_flow.rs`).
- Todas as 7 bibliotecas do catálogo **`F9`** (`macroquad`, `bevy 0.20`, `minifb`, `eframe`, `tachyonfx`, `rand`, `rodio`) trazem metadados estendidos (`V3LibraryMetadata`) com dependências de sistema Linux, lições V3 associadas, armadilhas comuns e limites ambientais documentados com honestidade.

---

## Itens que Dependem de Hardware / Ambiente Externo (Não Simulados)
- **Janelas GUI/3D nativas (`macroquad`, `minifb`, `eframe`, janela gráfica `bevy`):** requerem servidor gráfico X11/Wayland ou ambiente desktop Windows; em ambientes headless são validados os núcleos lógicos e testes unitários/headless.
- **Compilação cruzada Windows e áudio físico em PulseAudio/PipeWire:** requerem toolchain MinGW no host e dispositivo de som ativo; o fallback mudo é validado automaticamente.
