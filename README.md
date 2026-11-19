# RustLab 🦀

Aprenda Rust no terminal com lições, exercícios e uma mini-IDE.
O mentor é opcional: o guia local funciona sem serviço de IA.

## Começar

Instale [Rust stable](https://rustup.rs) com Cargo e use um terminal interativo.
A validação automatizada principal roda em Linux x86_64; outras plataformas
precisam de validação antes de serem anunciadas como suportadas.

```sh
git clone https://github.com/Honraan/RUSTLAB.git
cd RUSTLAB
cargo install --path . --locked
rustlab --doctor
rustlab
```

`--doctor` verifica Cargo, rustc, rust-analyzer, curl e a configuração do mentor
sem enviar código, consultar servidores ou mostrar tokens. Componentes opcionais
ausentes não impedem o guia offline. Cargo e rustc são necessários para executar
as práticas. Para habilitar o LSP: `rustup component add rust-analyzer`.
APIs HTTPS precisam de `curl` disponível no `PATH`.

Binários Linux serão anexados às [releases](https://github.com/Honraan/RUSTLAB/releases)
quando uma tag `v*` compatível com a versão do Cargo.toml for publicada e o workflow
concluir. O binário do app não substitui a instalação de Rust para os exercícios.

## Como usar

1. Abra **Aprender**, escolha uma lição e leia a explicação.
2. Responda ao quiz e use `e` nas lições com prática relacionada.
3. Edite a solução, execute os testes com `F5` e consulte as dicas com `F1`.
4. Confira **Meu Progresso**; use **Criar** para projetos próprios.

`Esc` volta, `Ctrl+C` sai e a ajuda mostra os atalhos de cada tela.
Executar projetos usa as permissões da sua conta; o executor não é uma sandbox.

## Disponível e planejado

- **Currículo inicial:** 30 lições em 10 etapas, 12 exercícios com testes Cargo,
  quizzes, widgets conceituais, progresso persistente e opções de acessibilidade.
- **Modo Criar:** editor, projetos Cargo, diagnósticos, LSP com fallback local e
  mentor opcional com Ollama ou API compatível com OpenAI.
- **Laboratório:** ferramentas pedagógicas de memória, depuração, macros e
  flashcards. Simulações e estimativas didáticas têm limitações; não substituem
  resultados do compilador ou de um depurador nativo. Os flashcards do laboratório
  mantêm acertos na sessão; não oferecem agendamento SRS persistente.
- **V3 em evolução:** blueprint de 213 entradas, traduções e prática integrada.
  Entrada planejada não equivale a aula publicada. Veja os estados e critérios em
  [CURRICULUM_V3.md](docs/CURRICULUM_V3.md).

O próximo foco é validar percursos completos de explicação, prática, feedback e
revisão com iniciantes antes de expandir o catálogo.

Veja o [inventário detalhado](docs/FEATURES.md), o
[roadmap V3](docs/V3_ROADMAP.md) e o [guia de contribuição](CONTRIBUTING.md).

## Rodando

Requer Rust stable (edição 2024) e um terminal interativo:

```sh
cargo run            # ou: cargo install --path .
cargo test           # unitários + snapshots de UI
python3 tests/tui_phase4.py target/release/rustlab  # após cargo build --release
python3 tests/tui_phase6.py target/release/rustlab  # smoke da mini-IDE em PTY
SNAPSHOTS_ATUALIZAR=1 cargo test --test ui_snapshot   # regravar snapshots
```

Config em `~/.config/rustlab/config.toml` (ou `RUSTLAB_CONFIG=/caminho`).

Documentação por fase: [docs/FASE4.md](docs/FASE4.md), [docs/FASE5.md](docs/FASE5.md),
[docs/FASE6.md](docs/FASE6.md), [docs/FASE7.md](docs/FASE7.md), [docs/FASE8.md](docs/FASE8.md) e
[docs/FASE9.md](docs/FASE9.md).

## Estrutura

```
src/
  main.rs      casca: ciclo de vida do terminal + panic hook
  lib.rs
  app.rs       máquina de telas, event loop, navegação (testável)
  event.rs     thread de leitura de teclado -> canal mpsc (não bloqueia a UI)
  config.rs    TOML com defaults, merge parcial e recuperação de arquivo ruim
  theme.rs     paletas semânticas (acento, sucesso, erro, bordas...)
  animation.rs wrapper do TachyonFX + MotionMode (full/reduced/off)
  audio.rs     tom suave de XP via paplay + detecção passiva de capacidade
  json.rs      parser e serializador JSON mínimo para LSP e APIs de IA
  lsp.rs       cliente rust-analyzer (JSON-RPC) + índice local de símbolos
  mentor.rs    mentor opcional (guia local determinístico, Ollama e API remota)
  i18n.rs      strings pt-BR centralizadas
  ui/          telas de aprendizagem, Modo Criar, exercícios e Mentor
  curriculum/  schema TOML + validação dos capítulos, lições e exercícios
  storage.rs   progresso do aluno, versionado e atômico
  exercises.rs editor, hints e harness Cargo temporário
  content/     capítulos, lições e exercícios TOML validados
tests/
  ui_snapshot.rs  snapshots por tela e por tamanho de terminal
  fx_finish.rs    animação termina E preserva o conteúdo (regressão real)
```


Licença: [GPL-3.0-only](LICENSE).
