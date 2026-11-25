# Notas de Lançamento — RustLab 4.0 (Windows Desktop Edition)

**Versão:** `4.0.0` · **Licença:** `GPL-3.0-only`

---

## O que há de novo no RustLab 4.0

1. **Aplicativo Desktop Nativo para Windows (Tauri v2 + React + CodeMirror 6 + xterm.js)**
   - Interface gráfica moderna em **português brasileiro (`pt-BR`)**, operando sem janela de console preta (`WINDOWS_GUI`), respeitando temas Claro, Escuro (Cuia), Sistema e Alto Contraste (`WCAG AAA`), Modo Foco (`Ctrl+Shift+F`), redução de animações, zoom independente de interface/editor e cues sonoros suaves em Web Audio API.
2. **Currículo Canônico de 213 Entradas Preservado + Persistência Compartilhada**
   - Os 6 estágios canônicos (**Básico, Meio, Difícil, Fim, Mini Paint, CaveGame**) e o núcleo pedagógico Rust continuam compartilhados diretamente entre a edição TUI Classic (`cargo run -p rustlab`) e o RustLab Desktop.
   - Lições teóricas, demonstrações passo a passo e quizzes funcionam **100% offline** mesmo antes de instalar o compilador Rust.
3. **Toolchain Doctor em 7 Etapas & Execução Cargo Real no Windows**
   - Inspeção honesta de `rustup`, `rustc`, `cargo`, `rustfmt`, `clippy`, `rust-analyzer`, linker MSVC (`cl.exe`/`link.exe`), Windows SDK e linker GNU/LLVM, com comandos copiáveis oficiais (nunca modifica o `PATH` do usuário nem solicita elevação UAC oculta).
   - Validação real de código via `cargo.exe` local com Windows Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`), diagnósticos clicáveis, explicações humanas do Ferris (`c(o.o)c` / `c(ovo)c` / `c(o,x,o)c` / `c(o,0)c`), bônus `100+100 XP` por independência e SRS.
4. **Mini-IDE Windows, Terminal ConPTY Interativo & Catálogo `F9`**
   - Criação de projetos a partir de 6 templates (`hello`, `pong`, `snake`, `editor`, `paint_gui`, `cavegame_bevy`), detecção de conflitos SHA-256, confiança explícita por projeto externo e terminal interativo **Windows ConPTY + `xterm.js`**.
   - Atalho `F9` preservado com prévia exata das alterações em `Cargo.toml`/`src/main.rs`, consentimento explícito e reversão em um clique.
   - Lançamento separado para projetos GUI (`F4`).
5. **Mentor Pedagógico Somente Leitura & Migração Segura (`TUI V3 -> V4`)**
   - No Modo Aprender, barreiras técnicas no backend impedem entrega de solução pronta ou alteração automática de arquivos.
   - Chaves de API remotas opcionais são armazenadas no **Windows Credential Manager** (nunca em `config.toml` ou backups).
   - Importação com prévia e backup automático para `progresso.toml` da TUI V3 e backups portáteis `.json` v4.

---

## Verificação e Limitações Transparentes desta Build

- **Build local verificada em:** Windows 10 Home x64 (Build `19045`), target `x86_64-pc-windows-gnullvm`, WebView2 `158.0.4370.33`.
- **Instalador NSIS gerado:** `RustLab_4.0.0_x64-setup.exe` (`5.18 MiB`, `5433162` bytes), SHA-256: `df7466320ade4af68d2f4752a9f00e4b201b14581e2a5184f8045701e7f543ee`.
- **SmartScreen:** como o instalador comunitário ainda não possui certificado EV / Azure Trusted Signing, o Windows Defender SmartScreen pode exibir aviso padrão até que o binário seja assinado em CI ou acumule reputação.
- **Execução local não é sandbox:** compilar ou executar programas Rust no host roda com as permissões da sua conta Windows; por isso o RustLab solicita consentimento explícito na execução e confiança por projeto externo.
- **Pendências documentadas de QA externo:** teste físico em **Windows 11 x64**, compilação local com **MSVC Build Tools** (coberta pelo workflow CI `windows-v4.yml`) e compilação/abertura visual com GPU real dos templates externos `paint_gui` (`eframe 0.31`) e `cavegame_bevy` (`bevy 0.20`).
