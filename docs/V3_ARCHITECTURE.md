# V3 Architecture — Contratos de Módulos, Adaptador Editor↔Lição e Migrações

## 1. Contratos de Responsabilidade Coesa

| Módulo / Responsabilidade | Arquivo(s) Atual(is) | Regra de Fronteira |
| --- | --- | --- |
| `app` | `src/app.rs` | Orquestra ciclo de vida, eventos, foco e persistência; nunca avalia corretude diretamente na UI. |
| `ui` | `src/ui/*.rs`, `src/theme.rs` | Renderiza estado imutável (ou scroll/viewport) com responsividade e fallback ASCII. |
| `learn` & `ferris` | `src/learn.rs`, `src/ui/lesson.rs` | Máquina de estados da lição (explicação acima, tarefa/quiz/prática abaixo, estados do Ferris). |
| `curriculum` | `src/curriculum/mod.rs`, `src/curriculum/v3_blueprint.rs` | Dados embutidos validados deterministicamente + blueprint V3 de 213 IDs e pré-requisitos. |
| `editor_adapter` | `src/editor.rs` + `LessonPractice` em `src/learn.rs` | Encapsula o componente `Editor` da mini-IDE sem duplicá-lo nem descartar rascunhos não salvos. |
| `project_workspace` | `src/projects.rs`, `src/create.rs` | Projetos Cargo reais no disco, criação em pastas escolhidas e detecção de capacidades. |
| `execution` | `src/execution.rs` | `Command::new("cargo")` assíncrono em Tokio com process group, timeout, cancelamento e limite de saída. |
| `assessment` & `hints` & `rewards` | `src/exercises.rs`, `src/learn.rs`, `src/app.rs` | Validação via `cargo test`/`cargo run` real; contagem idempotente de dicas e XP `100 + 100`. |
| `progress` | `src/storage.rs` | Escrita atômica segura (`.tmp` + rename + backup `.corrupt`) com compatibilidade retroativa. |
| `mentor` | `src/mentor.rs`, `src/ui/mentor.rs` | Modo Aprender restrito a professor somente-leitura sem código; Modo Criar mantém assistência completa. |
| `library_catalog` | `src/doc_hint.rs` | Catálogo `F9`, glossário offline `pt-BR` e inserção assistida em `Cargo.toml` / `main.rs`. |

## 2. Proposta e Implementação do Adaptador Editor↔Lição (V3.1)

Para cumprir o requisito V3.1 sem reescrever o editor existente:
1. `LessonPractice` dentro de `LessonView` reutiliza diretamente `crate::editor::Editor` (mesmo buffer de texto, cursor, seleção, desfazer/refazer, destaque sintático e estado `dirty`).
2. Ao abrir uma lição prática mapeada (como `B-P01 · hello-world` ou `B-P02 · variaveis`), o aluno lê a explicação no **Painel Superior** e pressiona `Tab` (ou `Shift+Tab`) para mover o foco ao **Painel Inferior**, onde edita código Rust real em `src/main.rs` (ou solução de exercício) enquanto a explicação permanece visível acima.
3. O salvamento de rascunhos da lição prática e o resultado do último `cargo run` / `cargo test` preservam o código escrito mesmo quando o estudante alterna painéis, abre dicas (`F1`) ou consulta outra tela.

## 3. Esquemas Versionados e Plano de Migração

### Esquema de Progresso (`progresso.toml`)
- **Atual (`version = 1`):**
  - `read_lessons`, `completed_lessons`, `completed_exercises`, `hints_used`, `exercise_answers`, `xp_events`, `project_milestones`.
- **Evolução planejada (`version = 2` compatível com v1):**
  - Manter todos os campos v1 preservados na deserialização (`#[serde(default)]`).
  - Registrar novos eventos de XP no mapa existente `xp_events` (`v3:practice:{id}:base`, `v3:practice:{id}:independent`) e respostas por ID estável em `exercise_answers` (ex.: `"v3:B-P01"`), garantindo migração sem perda para usuários da v1.
  - Futuros campos (histórico de tentativas sem dica, pontuação de domínio por `skill_tag` e revisão espaçada) serão adicionados de forma aditiva com `#[serde(default)]` em v2.
