# Checklist de Lançamento Windows — RustLab 4.0

**Versão:** `4.0.0` (`GPL-3.0-only`)  
**Host de verificação local:** Windows 10 Home x64 (Build `19045`), usuário sem privilégios de administrador (`IsAdmin: False`), toolchain `1.100.0-x86_64-pc-windows-gnullvm` + `llvm-mingw` portátil (`clang 23.1.6`) + Node.js `v22.23.5`, Microsoft Edge WebView2 `158.0.4370.33`.

---

## 1. Itens Verificados com Evidência Real neste Host

| Área | Verificação Executada | Resultado Observado |
|---|---|---|
| Formatação & Lint Rust | `cargo fmt --all -- --check` e `cargo clippy --workspace --all-targets -- -D warnings` | Aprovado (`exit 0`, zero warnings em `rustlab` e `rustlab-desktop`) |
| TUI Classic + Núcleo Compartilhado | `cargo test --workspace --all-targets` (crate `rustlab`) | **269 testes aprovados** (228 lib + 1 `cargo_run` + 2 `cli` + 7 `exercise_flow` + 1 `fx_finish` + 1 `project_flow` + 29 `ui_snapshot`; 1 teste de animação ignorado por tempo real ~2,5s) |
| Currículo Canônico 213 Entradas | `curso_tem_213_entradas_em_pt_br_e_bt01_real` (`backend.rs`) | Aprovado (6 estágios, 213 entradas reais, `B-T01` + quiz) |
| Compilação Real & XP `100+100` | `pratica_cargo_real_bp01_e_d09_com_100_mais_100_xp` (`backend.rs`) | Aprovado (recusa sem consentimento, compila `B-P01` real e concede `100+100 = 200 XP`; diagnostica `B-P02` + Ferris) |
| Templates Mini-IDE `cargo test` | `templates_std_compilam_e_passam_cargo_test_real` (`backend.rs`) | Aprovado (`hello`, `pong`, `snake`, `editor` compilam e passam `cargo test --offline`) |
| Terminal Windows ConPTY | `windows_conpty_real_executa_stdin_e_stdout` (`backend.rs`) | Aprovado (`portable-pty` ConPTY real com handshake `\x1b[1;1R` e saída `CONPTY_OK`) |
| Segurança de Caminhos & Allowlist URL | `seguranca_caminhos_unicode_e_allowlist_url` (`backend.rs`) | Aprovado (bloqueia `../`, `Program Files`, nomes não-ASCII e URLs fora da allowlist como `file:///`) |
| Catálogo `F9` + Reverter | `f9_preview_apply_e_revert_funcionam` (`backend.rs`) | Aprovado (preview, apply em `Cargo.toml`/`src/main.rs` e revert limpo) |
| Migração V3 & Backup | `migracao_backup_v3_e_idempotencia_xp` (`backend.rs`) | Aprovado (prévia `progresso.toml`, merge idempotente de XP, export `.json`) |
| Mentor Somente Leitura | `mentor_learn_mode_bloqueia_codigo` (`backend.rs`) e Vitest `MentorDrawer` | Aprovado (`readOnlyEnforced: true`, sem bloco de solução pronta no Modo Aprender) |
| Frontend TypeScript, Vitest & Vite | `npm run typecheck`, `npm test`, `npm run build` | Aprovado (0 erros TS, **5/5 testes Vitest**, bundle gerado em `apps/desktop/dist`) |
| Subsistema Windows GUI (Sem Console) | `llvm-objdump -p target/release/rustlab-desktop.exe` | Aprovado (`Subsystem 00000002 (Windows GUI)`) |
| Empacotamento NSIS + `WebView2Loader.dll` | `npx tauri build --bundles nsis` | Aprovado: `RustLab_4.0.0_x64-setup.exe` (**5.18 MiB**, SHA-256 `df7466320ade4af68d2f4752a9f00e4b201b14581e2a5184f8045701e7f543ee`) |
| Instalação & Desinstalação Silenciosa | `/S /D=<temp>` e `uninstall.exe /S` | Aprovado (`rustlab-desktop.exe`, `WebView2Loader.dll`, `uninstall.exe` instalados; desinstalação removeu EXEs mantendo arquivo sentinela de usuário intacto) |

---

## 2. Itens Pendentes de QA Externo Antes de Declarar Suporte Universal

- [ ] **Windows 11 x64 (`22H2+`):** execução interativa da janela principal WebView2, redimensionamento `1366×768` / DPI `100–200%` e sessão manual completa do Toolchain Doctor.
- [ ] **Toolchain `x86_64-pc-windows-msvc` + Visual Studio 2022 Build Tools + Windows 10/11 SDK:** validar em runner GitHub Actions (`windows-latest` via [windows-v4.yml](file:///c:/Users/drago/Documents/GitHub/RUSTLAB/.github/workflows/windows-v4.yml)) ou máquina Windows MSVC dedicada (o host local utilizou `x86_64-pc-windows-gnullvm` sem MSVC instalado).
- [ ] **Capstones Gráficos (`paint_gui` com `eframe 0.31` e `cavegame_bevy` com `bevy 0.20`):** neste host offline sem essas crates em cache, `cargo check --offline` retornou `exit 101` (`no matching package found`). Validar download crates.io + compilação + abertura da janela 2D/3D em hardware com driver OpenGL/DirectX 12/Vulkan compatível.
- [ ] **Assinatura de Código Authenticode (EV / Azure Trusted Signing):** instalador gerado está sem assinatura comercial; configurar segredos em [release-windows.yml](file:///c:/Users/drago/Documents/GitHub/RUSTLAB/.github/workflows/release-windows.yml) quando houver certificado disponível.
- [ ] **Matriz Ubuntu CI (`.github/workflows/ci.yml`):** executar `tests/tui_phase4.py` e `tests/tui_phase6.py` no runner Linux após merge.
