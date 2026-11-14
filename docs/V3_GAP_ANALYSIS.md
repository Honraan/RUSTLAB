# V3 Gap Analysis — Auditoria do Repositório RustLab

Auditoria baseada exclusivamente em arquivos, módulos, testes e resultados observados no workspace atual.

## 1. Recursos Existentes Comprovados (Mapa de Reuso)

| Capacidade | Localização verificada no código | Testes / Evidência existente | Plano de Reuso na V3 |
| --- | --- | --- | --- |
| UI TUI e tema adaptativo (4 temas, Unicode/ASCII, Reduced/Off motion) | `src/ui/mod.rs`, `src/theme.rs`, `src/config.rs`, `src/animation.rs` | `tests/ui_snapshot.rs` (27 snapshots golden), `animation::tests` | Preservar `Theme`, `AnimEngine` e degradar layouts responsivamente em `ui/lesson.rs` e `ui/exercises.rs`. |
| Editor de código real e destaque sintático | `src/editor.rs`, `src/highlight.rs`, `src/completion.rs` | `editor::tests`, `completion::tests` | Reutilizar `Editor` via `LessonPractice` em `src/learn.rs` e manter intacto no Modo Criar (`src/create.rs`). |
| Mini-IDE completa (árvore, múltiplos arquivos, `F5`/`F6`/`F7`/`F8`, `Ctrl+L`, `PLANO.md`, `F4`, `F9`, `Alt+A`) | `src/create.rs`, `src/ui/create.rs`, `src/projects.rs`, `src/doc_hint.rs` | `create::tests`, `projects::tests`, `tests/tui_phase6.py` | Conservar comportamento e atalhos existentes sem reescrever. |
| Executor Cargo assíncrono não-bloqueante sem shell | `src/execution.rs` (`ExecutionManager`, `ToolCommand`) | `execution::tests`, `tests/cargo_run.rs`, `tests/exercise_flow.rs` | Reutilizar para exercícios e prática integrada em lições; evoluir para PTY interativo real na V3.2/V3.4. |
| Catálogo atual de currículo e validador TOML | `src/curriculum/mod.rs`, `src/content/{chapter,lesson,exercise}/*.toml` | `curriculum::tests` (30 lições em 10 etapas, 12 exercícios com 4 dicas e testes reais) | Manter lições/exercícios atuais intactos e adicionar `v3_blueprint.rs` como camada de mapeamento/linter para 213 IDs. |
| Sistema de exercícios e persistência de rascunhos | `src/exercises.rs`, `src/storage.rs` (`progresso.toml` v1, escrita atômica `.tmp` + `.corrupt`) | `exercises::tests`, `storage::tests`, `tests/tui_phase4.py` | Preservar `drafts`, `exercise_answers`, `hints_used` e evoluir schema sem perder progresso. |
| Mentor IA + Seletor Ollama + Revisão periódica | `src/mentor.rs`, `src/ui/mentor.rs` | `mentor::tests` (mock HTTP Ollama/OpenAI, `/api/tags`, proteção `<dados_nao_confiaveis>`, autoria estudante vs IA) | Adicionar fronteira explícita de professor somente-leitura no Modo Aprender (`is_learn_mode_context` + sanitização de blocos de código). |
| Áudio opcional | `src/audio.rs` | `audio::tests` | Manter detecção opcional (`paplay`), silêncio sem hardware e sem clique por tecla. |

## 2. Lacunas Documentadas Frente à Especificação V3

| Lacuna | Impacto na V3 | Fase Alvo |
| --- | --- | --- |
| Painel único contínuo em `ui/lesson.rs` sem separação fixa Explicação (topo) × Quiz/Prática (base) e sem Ferris reativo à direita. | Dificulta leitura de texto longo e prática inline sem sair da lição. | **V3.1** (resolvido nesta entrega). |
| Prática `B-P01` (`Hello, world!`) ainda não abria editor Cargo embutido direto no painel inferior da lição. | Aluno precisava sair para outra tela ou criar projeto avulso. | **V3.1** (adaptador `LessonPractice` + execução Cargo real). |
| IA no `Screen::Mentor` não bloqueava explicitamente blocos de código em respostas quando aberta a partir do Modo Aprender. | Risco de entregar resposta pronta no Modo Aprender. | **V3.1** (`sanitize_learn_mode_response` + prompt professor). |
| Bônus de independência `100 + 100 XP` ainda usava pontuação antiga fixa (20/30 XP) sem distinção no ledger para prática V3. | Falta recompensa transparente por autoria sem dicas opcionais. | **V3.1** (`calculate_v3_practice_xp` e suporte no ledger). |
| Faltam os 183 conteúdos completos além das 30 lições atuais e 12 exercícios atuais (total alvo = 213 entradas). | Catálogo completo está especificado no blueprint, com autoria faseada honesta. | **V3.2–V3.8**. |
| Execução interativa CLI usa pipes (`RunInteractive`) em vez de PTY real (`openpty`/termios) e falta matriz automatizada completa de janelas GUI / `x86_64-pc-windows-gnu`. | Impede testar jogos TUI real-time (Pong/Snake/editor) via PTY no momento. | **V3.2 / V3.4 / V3.7 / V3.8**. |

## 3. Riscos, Desconhecidos e Próximo Incremento Verificável

- **Risco 1:** Alterar pontuação de exercícios legados sem cuidado quebraria `tests/exercise_flow.rs` (`assert_eq!(app.xp(), 30)` quando 1 dica `F1` foi usada). Mitigação: preservar evento `30 XP` quando `hints_used > 0` em exercícios legados e aplicar bônus de independência `+100` apenas quando concluído sem dicas/IA (ou na prática integrada V3.1).
- **Risco 2:** Alterar `tests/snapshots/*.txt` sem preservar atalhos usados por `tui_phase4.py` (`e` para exercícios, `F5`, `F1`, `Esc`). Mitigação: manter todos os atalhos e rodar `tui_phase4.py` e `tui_phase6.py` após cada mudança.
- **Primeiro incremento codificado na V3.0/V3.1:** `src/curriculum/v3_blueprint.rs` (213 entradas + validador), redesign de `ui/lesson.rs` + `ui/exercises.rs` com Ferris `c(o.o)c`/`c(ovo)c`/`c(o,x,o)c`/`c(o,0)c`, `LessonPractice` com `Editor` real no painel inferior e trava read-only em `src/mentor.rs`.
