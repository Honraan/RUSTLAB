# V4 Roadmap — Plano de Entrega em Fatias Verticais (Windows Desktop Edition)

**Escopo:** Exclusivamente **Windows Desktop** (Tauri v2 + React/TS/Vite sobre núcleo Rust compartilhado), mantendo a **TUI Classic** funcional e o currículo canônico de **213 entradas**.

---

## 1. Visão Geral das Fases V4.0 – V4.8

| Fase | Foco Principal | Entregáveis-Chave | Critério de Saída ("Done When") |
|---|---|---|---|
| **V4.0** *(Atual)* | Auditoria real, ADR, Contrato Windows, Migração, Modelo de Ameaças e Roadmap | `docs/V4_BASELINE_AUDIT.md`, `V4_ARCHITECTURE.md`, `V4_WINDOWS_CAPABILITIES.md`, `V4_DATA_MIGRATION.md`, `V4_THREAT_MODEL.md`, `V4_ROADMAP.md` | Estado real documentado sem suposições; módulos portáveis mapeados; fatia V4.1 acordada; TUI intacta. |
| **V4.1** | Piso de portabilidade Windows + Casca Tauri v2 + 1ª fatia vertical + 1º NSIS `.exe` | Compilação verde no Windows, janela sem console, Home + Mapa (213 entradas reais) + Leitor `B-T01` + Ferris + Sobre/Diagnóstico + CI Windows gerando `-setup.exe` | App Windows instalado abre sem terminal; lição real do núcleo compartilhado renderiza; TUI segue funcional. *(Não declarar IDE completa).* |
| **V4.2** | UI completa do currículo (213 entradas), Prática integrada e Migração segura | Painel Explicação × Quiz/Prática em `pt-BR`, gating DAG real, persistência `progresso.toml` v1/v2 com backup e import/export, código editável preservado no CodeMirror 6 | Amostras de todos os 6 estágios renderizam conteúdo real; progresso e código sobrevivem à reabertura; migração testada. |
| **V4.3** | Mini-IDE Windows real, Cargo, Toolchain Doctor e ConPTY interativo | Árvore de arquivos, abas, salvamento seguro, `cargo check/run/test/fmt/clippy`, diagnósticos `C:\...` clicáveis, Job Objects, ConPTY + `xterm.js`, Toolchain Doctor 7 passos | Compilação, execução, testes, `stdin`/PTY e validação determinística (`100+100 XP`) funcionando de ponta a ponta no Windows real. |
| **V4.4** | Catálogo `F9`, Capstones GUI separados (`Mini Paint`, `CaveGame`) e fluxos avançados | Painel `F9` (atalho `F9` preservado) com preview/consentimento, revisão de versões (`tachyonfx`), janelas GUI separadas para `Paint` e `Bevy 0.20`, ConPTY polido para Pong/Snake/Editor, SRS e Mentor read-only | Adição de crate verificada; `Mini Paint` preserva desenho no resize; `CaveGame` abre janela 3D em GPU compatível; ciclo completo de XP/SRS/IA testado no Windows. |
| **V4.5** | Polimento visual/sonoro e acessibilidade desktop | Web Audio cues suaves, animações Ferris, celebrações de capítulo, Modo Foco, Alto Contraste, zoom independente UI/editor, 1366×768 + DPI 100–200%, operação 100% teclado/ARIA | UI acolhedora sem poluição; todos os fluxos críticos operáveis só por teclado, sem som e com movimento reduzido. |
| **V4.6** | Hardening de instalador, segurança, recuperação de crash e CI release | QA instalação limpa, WebView2 ausente/presente, sem/com toolchain, file locks/antivírus, crash drafts, `cargo audit`, política de atualização assinada, release notes/licenças | `.exe` reproduzível validado em matriz Windows; zero segredos expostos e migrações não destrutivas comprovadas. |
| **V4.7** | Beta fechado e fidelidade pedagógica | Testes com iniciantes e usuários Rust reais; correção de atritos de onboarding, mensagens do compilador, clareza de lições, cálculo de XP e capstones | Iniciante entra, entende, edita, compila, vê saída real, corrige erros e avança com autonomia sem ajuda externa. |
| **V4.8** | Lançamento estável Windows e manutenção | Matriz final de regressão Desktop + TUI, release tagueada no GitHub com `-setup.exe` + `SHA256SUMS` + instruções/limitações honestas e backlog de manutenção | Aplicativo Windows distribuível, mantível e verificado sem nenhum recurso simulado apresentado como pronto. |

---

## 2. Especificação Precisa da Próxima Fatia: V4.1 (Escopo Estreito e Executável)

A V4.1 **não** tenta entregar o IDE completo nem os capstones gráficos. Seu único objetivo é colocar de pé uma **casca Windows real, instalável e conectada ao currículo autêntico**, mantendo a TUI Classic 100% intacta.

### 2.0 Pré-requisitos de Ambiente (Exigem Consentimento Explícito — Regra 11)
No host auditado (Windows 10 Home 19045), antes de compilar ou empacotar V4.1 localmente será necessário autorizar a instalação de:
1. Toolchain Rust `x86_64-pc-windows-msvc` (`rustup toolchain install stable-msvc` / `default`);
2. Visual Studio 2022 Build Tools (**Desktop development with C++** + **Windows 10/11 SDK**);
3. Node.js LTS (para Vite/React/TypeScript e `@tauri-apps/cli`);
4. NSIS (baixado pelo bundler do Tauri ou instalado no sistema) — ou execução alternativa da etapa de build do instalador via runner GitHub Actions `windows-latest`.

### 2.1 Subpasso V4.1-A — Piso de Portabilidade Windows (Núcleo + TUI)
- Isolar com `#[cfg(unix)]` / `#[cfg(windows)]` apenas o necessário para compilar e passar testes no Windows:
  - `src/storage.rs`: substituir `OpenOptionsExt::mode` no Windows; resolver `%LOCALAPPDATA%\RustLab\progresso.toml` em `#[cfg(windows)]`.
  - `src/config.rs`: resolver `%APPDATA%\RustLab\config.toml` e `%LOCALAPPDATA%\RustLab\target-cache` em `#[cfg(windows)]`.
  - `src/projects.rs`: resolver `USERPROFILE` em `expand_home`, `%USERPROFILE%\Documents\RustLab Projetos` em `default_workspace_dir`, normalizar separadores nos testes e substituir `std::os::unix::fs::symlink` nos testes Windows.
  - `src/execution.rs`: isolar `PermissionsExt`/`CommandExt`/`killpg` em `#[cfg(unix)]` e implementar stubs/base limpa `#[cfg(windows)]` com `cargo.exe` + `CREATE_NO_WINDOW` + `child.kill()` (Job Objects completos entram na V4.3).
  - `src/audio.rs`: manter `#[cfg(unix)]` para `paplay`/ALSA e fallback silencioso limpo no Windows TUI.
- **Não alterar** o comportamento funcional da TUI Linux (`tests/tui_phase4.py`, `tests/tui_phase6.py`, `tests/ui_snapshot.rs`).

### 2.2 Subpasso V4.1-B — Shell Tauri v2 + UI Inicial + 1 Lição Real
- Estruturar workspace Cargo mantendo `rustlab` (TUI) compilável na raiz e adicionando `apps/desktop` (`src-tauri` + React/TypeScript/Vite).
- Configurar `tauri.conf.json`:
  - Título `RustLab`, ícone oficial do app, janela inicial adaptativa (mínimo `1024×680`, testado em `1366×768`), sem console no Windows.
  - `bundle.windows.webviewInstallMode: { "type": "embedBootstrapper" }`.
  - `capabilities/main.json` mínimo e CSP restrita.
- Implementar bridge IPC mínima da V4.1:
  - `get_windows_capabilities()` (relatório honesto de SO, WebView2, Cargo/rustc/MSVC).
  - `get_v3_course_overview()` (6 estágios + 213 entradas reais do `v3_blueprint`, em `pt-BR` por padrão).
  - `get_lesson(entry_id)` (retornando lição real gerada por `build_v3_lesson_for` + `Curriculum::embedded_for`, começando pela fatia vertical `B-T01` *"O que é programar?"* e leitura estrutural de `B-P01`).
- Telas da V4.1 no frontend React:
  - **Início:** boas-vindas em `pt-BR`, card "Continuar aprendendo (`B-T01`)", resumo dos 6 estágios e Ferris `c(o.o)c`.
  - **Curso:** lista navegável dos 6 estágios e 213 entradas reais (marcando claramente teoria vs. prática e o que está ativo na V4.1 vs. previsto para V4.2/V4.3).
  - **Leitor de Lição (Vertical Slice):** renderiza `B-T01` com explicação à esquerda, bloco de código, demo interativa passo a passo, quiz determinístico funcional e reações do Ferris (`c(o.o)c` / `c(ovo)c` / `c(o,x,o)c`).
  - **Configurações básicas & Ajuda/Sobre:** tema Claro/Escuro/Sistema, animação reduzida, idioma `pt-BR`/`en-US`, licença `GPL-3.0-only`, diagnóstico honesto do sistema.

### 2.3 Subpasso V4.1-C — Primeiro Instalador NSIS e CI Windows
- Criar workflow `.github/workflows/windows-v4.yml` em `windows-latest`:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `npm ci && npm run typecheck && npm run build` (em `apps/desktop`)
  - `tauri build --bundles nsis` gerando `RustLab_<versao>_x64-setup.exe` + `SHA256SUMS`.
- Preservar `.github/workflows/ci.yml` (Ubuntu) rodando toda a regressão da TUI Classic.

---

## 3. Critérios de Aceitação e Testes da V4.1

A fase **V4.1** só poderá ser marcada como concluída quando **todos** os itens abaixo forem verificados com saída real:

1. **Regressão Rust & TUI intacta:**
   - `cargo fmt --all -- --check` passa (`exit 0`).
   - No Windows (`x86_64-pc-windows-msvc`): `cargo clippy --workspace --all-targets -- -D warnings` e `cargo test --workspace` passam sem falhas.
   - No Linux CI (`ubuntu-latest`): `cargo test --locked`, `tests/tui_phase4.py` e `tests/tui_phase6.py` continuam 100% verdes.
2. **Frontend & Contrato IPC:**
   - Typecheck TypeScript (`tsc --noEmit`), lint e build Vite passam sem erros.
   - Teste de contrato garante que os 213 IDs e o conteúdo de `B-T01` exibidos na UI vêm diretamente do `Curriculum::embedded()` / `v3_blueprint` (zero cópia JSON estática paralela).
3. **Experiência Desktop Windows real:**
   - Instalador NSIS `*-setup.exe` instala em máquina/VM Windows limpa, cria atalho no Menu Iniciar e **abre a janela gráfica sem nenhum console preto aberto**.
   - Redimensionamento para `1366×768`, alternância de tema e operação por teclado (`Tab`, setas, `Enter`, `Esc`) funcionam sem travamentos.
   - Lição `B-T01` exibe texto, demo interativa e quiz com reação do Ferris; tela *Sobre / Diagnóstico* exibe estado honesto de Rust/WebView2.
   - Desinstalação remove o aplicativo sem apagar arquivos do usuário e sem erros.
   - **Proibido na V4.1:** declarar que o mini-IDE ou execução Cargo de lições práticas está completo (pertence à V4.3).

---

## 4. Mapeamento da Definição de Pronto do Produto (14 Critérios → Marcos)

| # | Expectativa do Usuário Windows (Seção 14) | Marco Responsável |
|---|---|---|
| 1 | Baixar e instalar sem saber Cargo/Git/Node/terminal | **V4.1** (1º NSIS) → **V4.6** (hardening) |
| 2 | Abrir pelo Menu Iniciar em `pt-BR` acolhedor | **V4.1** |
| 3 | Ler lições, responder quizzes e salvar progresso sem Rust instalado | **V4.1** (leitura `B-T01`) → **V4.2** (213 + persistência) |
| 4 | Seguir setup guiado de Rust + MSVC Build Tools com consentimento | **V4.3** (Toolchain Doctor completo) |
| 5 | Editar Rust real, compilar e ver saída verdadeira no RustLab | **V4.3** |
| 6 | Ver erros do compilador com explicação humana e corrigir sozinho | **V4.3** (offline) + **V4.4** (AI mentor read-only) |
| 7 | Usar `F9` para entender/adicionar bibliotecas em projetos | **V4.4** |
| 8 | Completar práticas com testes determinísticos e bônus `100+100 XP` | **V4.2** (contrato) → **V4.3** (execução real) |
| 9 | Rodar programas e jogos interativos no terminal (stdin/setas) via ConPTY | **V4.3** (ConPTY) → **V4.4** (Pong/Snake/Editor) |
| 10 | Rodar `Mini Paint` e `CaveGame` (`bevy`) em janelas Windows separadas | **V4.4** |
| 11 | Salvar, reabrir, importar e exportar projetos Cargo e progresso | **V4.2** (progresso) + **V4.3** (projetos) + **V4.6** |
| 12 | Estudar offline / sem IA com avisos honestos quando faltar toolchain | **V4.2** – **V4.4** |
| 13 | Operar por teclado, sem som ou com movimento reduzido | **V4.1** (base) → **V4.5** (polimento WCAG) |
| 14 | Desinstalar sem destruir projetos do estudante | **V4.1** (regra NSIS) → **V4.6** (QA de desinstalação) |

---

## 5. Estado Atual Verificado (Windows 10 Build `19045` · `x86_64-pc-windows-gnullvm`)

- **Implementado no workspace:** núcleo compartilhado `rustlab` portável para Windows + TUI Classic preservada; casca desktop Tauri v2 (`apps/desktop/src-tauri`) + frontend React/TS/CodeMirror 6/`xterm.js` (`apps/desktop/src/`); Toolchain Doctor 7 passos; executor Cargo com Job Objects; ConPTY interativo; Mini-IDE com detecção de conflitos SHA-256; catálogo `F9` com prévia/reversão; Mentor read-only com Windows Credential Manager; export/import `.json` e `progresso.toml`; workflows `.github/workflows/windows-v4.yml` e `.github/workflows/release-windows.yml`.
- **Verificado localmente nesta máquina:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-targets` (`rustlab` 269 testes + `rustlab-desktop` 8 testes de integração), Vitest (`5/5`), Vite build, geração NSIS (`RustLab_4.0.0_x64-setup.exe`, `5.18 MiB`, SHA-256 `df7466320ade4af68d2f4752a9f00e4b201b14581e2a5184f8045701e7f543ee`), bundling automático de `WebView2Loader.dll` e instalação/desinstalação silenciosa sem destruir arquivos do usuário.
- **Pendente de QA externo:** ver [V4_WINDOWS_RELEASE_CHECKLIST.md](V4_WINDOWS_RELEASE_CHECKLIST.md) (Windows 11 x64 físico, build local `x86_64-pc-windows-msvc`, compilação/display real de `paint_gui`/`cavegame_bevy` e assinatura Authenticode).
