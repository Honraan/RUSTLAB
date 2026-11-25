# Checklist do instalador e da release Windows

## Verificado no commit da prévia

- [x] Windows Server 2025 x64 em runner hospedado. Clippy, integração Cargo, diagnóstico e ConPTY passaram. O teste ConPTY compilou um projeto em caminho com espaços e acento, confirmou console real, stdin, tecla de seta e saída ANSI.
- [x] NSIS x64 gerado com lockfiles, licença e SHA-256. Instalação e abertura confirmadas. A automação abriu `B-T01`, confirmou 20 XP no TOML, reabriu o aplicativo, confirmou a retomada e desinstalou.
- [x] Mini Paint 0.27.0 compilou e abriu em janela nativa no runner. O teste do canvas preservado em redimensionamento e exportação PPM passou.
- [x] CaveGame com Bevy 0.20.1 passou em `cargo check --all-targets` no Windows. A execução com GPU e o percurso completo de interação estão na matriz manual.
- [x] TUI Classic em Linux passou formatação, Clippy, build, 268 testes e os dois smokes PTY. Um teste de animação fica ignorado pela suíte.

| Comando | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | Passou |
| `cargo clippy --locked --all-targets -- -D warnings` | Passou em Linux e Windows |
| `cargo test --locked --all-targets` | Passou em Linux, com um teste ignorado |
| `cargo build --release --locked` | Passou em Linux |
| `python3 tests/tui_phase4.py target/release/rustlab` e `python3 tests/tui_phase6.py target/release/rustlab` | Passaram em PTY Linux |
| `cargo test --locked --test cargo_run --test project_flow --test exercise_flow` | Passou no Windows |
| `cargo test --locked --lib conpty_cargo_uses_a_console_and_delivers_keyboard_input` | Passou no Windows |
| `cargo test --locked --manifest-path apps/desktop/src-tauri/Cargo.toml` | Seis testes passaram em Linux e Windows |
| `npm --prefix apps/desktop run build` | TypeScript e Vite passaram |
| `npm --prefix apps/desktop audit --omit=dev` | Zero vulnerabilidades reportadas |
| `npm run tauri -- build --ci --target x86_64-pc-windows-msvc --bundles nsis -- --locked` | NSIS gerado e validado |

O instalador corresponde ao commit `0296a2a0429239b52d8d2c11f78a907c02dd0fc2`. SHA-256 do executável `279f8b94b1265953d9bf2a78bf990eeba81c59ed08575495c7d509f107421e55`. [CI Windows](https://github.com/Honraan/RUSTLAB/actions/runs/45686970497) · [CI Classic](https://github.com/Honraan/RUSTLAB/actions/runs/45686970452) · [Artefato NSIS](https://github.com/Honraan/RUSTLAB/actions/runs/45686970497/artifacts/15057137391)

## Windows 11 x64 em máquina ou VM

- [ ] Instalar como usuário comum com WebView2 existente e, em outra imagem, verificar a instalação do bootstrapper.
- [ ] Verificar nome e ícone dos atalhos, ausência de janela de console ao abrir e desinstalação/reinstalação com progresso e projetos preservados.
- [ ] Abrir sem Cargo/Rust e sem rede. Ler `B-T01`, acertar quiz e reabrir para conferir XP. Testar os estados de toolkit ausente, linker/SDK ausentes e toolkit completo.
- [ ] Usar caminhos com espaços, acentos, unicode e nomes longos. Exercitar arquivos travados, rascunho após travamento, conflitos externos e progresso corrompido/versão futura.
- [ ] Executar projeto com stdin e tecla de seta, resize e cancelamento. Verificar árvore de processos sem órfãos. Executar Pong/Snake/editor interativos e registrar os binários usados.
- [ ] Mini Paint: desenhar, redimensionar várias vezes, desfazer/refazer e comparar pixels no PPM exportado.
- [ ] CaveGame: registrar Windows build, driver, GPU e renderer. Testar movimento, salto, mouse, colocação/quebra de blocos e janela separada.
- [ ] Ativar alto contraste, testar Narrator, IME, teclado, foco, som desligado, monitor secundário e DPI 100%, 125%, 150% e 200%.
- [ ] Realizar sessões beta representativas e registrar correções. Windows 10 e ARM64 exigem matriz específica antes de serem anunciados.

## Distribuição

Para artefatos de prévia, use o arquivo do workflow **Desktop Windows** ligado ao SHA verificado e compare com `Get-FileHash instalador-setup.exe -Algorithm SHA256`. O executável de prévia está sem assinatura e o SmartScreen pode exibir aviso. Documente WebView2, versão de Rust/SDK para compilar, requisitos de GPU dos exemplos e política de desinstalação. Publicação estável exige autorização para release/tag, registro de proveniência, notas de versão, revisão de licenças transitivas e validação da estratégia de assinatura. [GitHub Actions](https://github.com/Honraan/RUSTLAB/actions/workflows/desktop-windows.yml)
