# Contribuir

Use Rust stable, Cargo, rustfmt e Clippy. O fluxo principal é validado em Linux.

```sh
rustup component add rustfmt clippy
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 tests/tui_phase4.py target/debug/rustlab
python3 tests/tui_phase6.py target/debug/rustlab
```

As práticas executam programas com as permissões da conta. Os smoke tests usam
configurações e progresso temporários. Não atualize snapshots para esconder uma
regressão; confira as diferenças visuais antes de aceitar novos resultados.

Para conteúdo, siga [CONTENT_AUTHORING_GUIDE.md](docs/CONTENT_AUTHORING_GUIDE.md).
Priorize percursos completos e teste explicação, prática, feedback e revisão com
iniciantes. Não marque conteúdo planejado como publicado.

Separe mudanças por responsabilidade: transporte de IA em `src/mentor/`, lógica
de edição em `src/editor.rs`, renderização em `src/ui/`. Prefira extrações graduais
com testes existentes preservados a reestruturações extensas em um único PR.

Antes de criar uma tag `vX.Y.Z`, alinhe a versão em Cargo.toml e Cargo.lock,
confira a CI e as notas de versão. O workflow gera um rascunho de release Linux
x86_64 com arquivo e checksum. Revise o rascunho antes da publicação.
