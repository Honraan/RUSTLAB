# Fase 9 — Currículo completo (Etapas 1–10), widgets visuais e Etapa 10

## Aceite do escopo

| Requisito | Entrega / evidência |
| --- | --- |
| Currículo completo Etapas 1–10 em pt-BR | `src/content/chapter/etapa-1.toml` a `etapa-10.toml` todas `status = "available"`, totalizando **30 lições** progressivas do "Olá, mundo!" até propriedade, borrowing, tipos/traits/módulos, `Option`/`Result`/testes, iteradores/closures/arquivos, CLI/async/JSON, projetos guiados e projeto independente. |
| 12 exercícios determinísticos com Cargo | `src/content/exercise/*.toml` (`soma`, `dobro`, `media`, `paridade`, `maior`, `intervalo`, `contar-vogais`, `filtrar-positivos`, `primeira-palavra`, `retangulo-area`, `divisao-segura`, `soma-quadrados-pares`), validados contra esqueletos e soluções reais em `tests/cargo_run.rs`. |
| Widgets visuais conceituais | `DemoKind` ampliado em `src/curriculum/mod.rs` e renderizado em `src/ui/lesson.rs` para `Variables`, `Loop`, `Function`, `StackHeap`, `Ownership`, `Borrowing` e `Vector`, com modo Unicode e fallback ASCII além do aviso explícito de simplificação didática (`t("demo.simplified")`). |
| Etapa 10 e conclusão por capacidade | No Modo Criar, `Alt+M` cria `PLANO.md` sem sobrescrever (`Project::write_project_plan_template`); execuções Cargo bem-sucedidas inspecionam capacidades reais (`ProjectCapabilities`: `PLANO.md`, multi-arquivo, `struct`/`enum`, `Option`/`Result`, docs `///`, build, `clippy`, `#[test]`, execução) e alimentam o painel `PROJETO INDEPENDENTE (ETAPA 10)` em `Meu Progresso`. |
| Marcos reconciliados (8 marcos) | `App::achievement_count`, `App::unlocked_by_event` e `src/ui/progress.rs` usam exatamente os mesmos 8 marcos, diferenciando conquistas do curso, exercícios e conclusão do projeto independente (apoiada por `plan`, `build`, `clippy`, `tests` e `run`, não apenas XP). |
| Polimento visual e ajuda atualizada | 4º tema `ThemeId::Oceano` em `src/config.rs` e `src/theme.rs`; tela de ajuda (`src/ui/help.rs`) e snapshots golden atualizados (`tests/snapshots/*.txt`). |

## Verificação reproduzível

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
python3 tests/tui_phase4.py target/release/rustlab
python3 tests/tui_phase6.py target/release/rustlab
```

## Limites explícitos

- As demonstrações visuais nas lições são diagramas conceituais interativos orientados a passos, não um depurador passo a passo (DAP/LLDB) do processo em tempo de execução.
- A detecção automática de capacidades da Etapa 10 analisa estrutura e marcadores explícitos nos `.rs` do projeto combinado ao sucesso dos comandos `cargo check`/`build`/`clippy`/`test`/`run`.
- Os testes PTY `tests/tui_phase4.py` e `tests/tui_phase6.py` cobrem em terminal real os fluxos principais de exercício e mini-IDE; o Mentor, o Agente lateral, o seletor de modelos Ollama e a seleção de pasta personalizada são cobertos pelos testes unitários em `mentor::tests`, `create::tests` e `projects::tests`, além dos snapshots `mentor_90x30` e `create_agente_90x30`.
