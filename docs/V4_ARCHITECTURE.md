# V4 Architecture — Decisões Arquiteturais (ADR), Workspace e Contratos IPC

**Status:** Especificação V4.0 (Windows Desktop Edition + preservação da TUI Classic)  
**Base verificada:** [`V4_BASELINE_AUDIT.md`](./V4_BASELINE_AUDIT.md)

---

## 1. Visão Geral: Duas Edições sobre Um Núcleo Único

O RustLab V4 mantém duas edições derivadas da **mesma fonte autoritativa** de currículo, regras pedagógicas, avaliação determinística e persistência:

1. **RustLab Desktop para Windows (Flagship V4):** shell Tauri v2 + backend Rust + frontend React/TypeScript/Vite rodando em Microsoft Edge WebView2, com instalador NSIS (`-setup.exe`), ConPTY interativo e lançamento de janelas GUI/3D nativas do estudante.
2. **RustLab TUI Classic (Retida):** aplicativo terminal Ratatui/Crossterm preservado sem reescrita destrutiva, consumindo os mesmos dados TOML e o mesmo núcleo educacional.

```text
           src/content/*.toml + v3_blueprint (213 IDs) + en_tr
                                   |
                   Núcleo Educacional Rust (Shared Core)
             (currículo, DAG, avaliação, XP 100+100, SRS,
              dicas, mentor read-only, storage, projetos)
                     /                           \
        Adaptador TUI Classic             Backend Windows Desktop
         (Ratatui/Crossterm)               (Tauri v2 + ConPTY +
          [preservado]                      Job Objects + LSP)
                                                  |
                                     Bridge IPC Tipado (Commands/Events)
                                                  |
                                      Frontend React + TypeScript
                                    (CodeMirror 6 + xterm.js + WebAudio)
                                                  |
                                     Processos Locais do Estudante
                                (cargo.exe / rustc.exe / ConPTY / GUI apps)
```

---

## 2. Registros de Decisão Arquitetural (ADRs)

### ADR-001: Stack do Aplicativo Desktop Windows
**Decisão:** Adotar **Tauri v2 + Backend Rust em Cargo Workspace + React 19 / TypeScript / Vite** usando **Microsoft Edge WebView2**.

| Critério | Tauri v2 + React/TS/Vite (Escolhido) | GUI Nativa Rust (`eframe` / `iced` / `slint`) | Electron (Chromium + Node) |
|---|---|---|---|
| Reuso do core Rust | **Direto** (mesmo binário/workspace Rust) | Direto | Indireto (exige IPC CLI ou N-API) |
| Acessibilidade Windows (UIA, leitores de tela, foco, alto contraste) | **Alta** (HTML semântico + ARIA nativo via WebView2) | Limitada/incompleta em vários widgets complexos | Alta |
| Editor de código rico + terminal VT100 | **Maduro** (CodeMirror 6 + `xterm.js`) | Exigiria construir editor e emulador VT100 quase do zero | Maduro |
| Tamanho do instalador Windows | **Enxuto** (~10–20 MB base + ~1,8 MB `embedBootstrapper`) | Enxuto (~10–25 MB) | Pesado (~150+ MB com Chromium) |
| Segurança de fronteira UI↔SO | **Explícita** (Capabilities + CSP + IPC tipado) | Sem fronteira web, porém menor ecossistema UI | Maior superfície Node/Chromium |
| Empacotamento Windows | **Nativo** (NSIS `-setup.exe` oficial Tauri 2) | Manual (WiX/NSIS avulsos) | Maduro (Squirrel/NSIS) |

**Justificativa:** Um aplicativo educacional de programação exige simultaneamente acessibilidade real (teclado + leitor de tela), layout dividido estável em 1366×768 com zoom independente para UI e código, editor com diagnósticos/inlay hints/seleção e um terminal interativo capaz de rodar jogos ANSI (`Pong`, `Snake`, `Mini Editor`). Tauri v2 maximiza o reuso do backend Rust já existente (`~42 mil` linhas) enquanto entrega UI moderna e instalador NSIS nativo sem embutir 150 MB de navegador.

---

### ADR-002: Editor de Código Embutido (CodeMirror 6 vs. Monaco)
**Decisão:** Adotar **CodeMirror 6** (`@codemirror/view`, `@codemirror/state`, `@codemirror/lang-rust`, `@codemirror/lint`, `@codemirror/autocomplete`, `@codemirror/search`, `@codemirror/commands`).

| Critério | CodeMirror 6 (Escolhido) | Monaco Editor |
|---|---|---|
| Peso no bundle local | ~150–250 KB gzip | ~2,5–4 MB + Web Workers |
| Instanciação em painéis divididos (Lição + Prática) | Instantânea, múltiplas instâncias leves | Mais pesada em memória e startup |
| Compatibilidade com CSP rígida no WebView2 | **Simples** (sem workers obrigatórios em blob/URL) | Requer configuração extra de workers sob CSP |
| Acessibilidade e zoom independente | Excelente suporte ARIA, wrapping e escala por CSS | Bom, porém layout interno mais rígido |
| Integração com `rust-analyzer` + `WorkspaceIndex` | Modular (`lintGutter`, `autocompletion`, `hoverTooltip`) | Rica, mas acoplada ao modelo próprio de serviços |

**Justificativa:** CodeMirror 6 oferece UX de IDE real (números de linha, realce Rust, parênteses pareados, multi-cursor, busca/substituição, undo/redo, marcadores de diagnóstico e inlay hints) mantendo inicialização rápida no Windows e zero conflito com CSP estrita.

---

### ADR-003: Execução Local Cargo, PTY Interativo e Janelas GUI Separadas
**Decisão:**
1. **Tarefas Cargo estruturadas** (`check`, `build`, `test`, `fmt`, `clippy`, `add`, `remove`, validação de exercícios): executadas no backend Rust via `tokio::process::Command` apontando para o caminho absoluto resolvido de `cargo.exe` + `Vec<String>` (`argv`), com `CREATE_NO_WINDOW` (`0x08000000`) e vinculadas a um **Windows Job Object** configurado com `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` para que cancelamentos (`Ctrl+K`/botão Parar) e timeouts encerrem `cargo.exe`, `rustc.exe` e qualquer processo filho sem órfãos.
2. **Terminal interativo real:** usar **Windows ConPTY** (via `portable-pty` v0.9+ ou wrapper Win32 equivalente verificado na V4.3) acoplado a **`xterm.js`** (`@xterm/xterm` + `@xterm/addon-fit`) no frontend. Substitui o antigo `RunInteractive` baseado em pipes por linha para exercícios com entrada raw/setas (`D09`), `Pong` (`D10`), `Snake` (`D11`), `Mini Editor` (`D12`) e console do Modo Criar.
3. **Aplicativos GUI/3D do estudante (`Mini Paint`, `CaveGame`/Bevy, `macroquad`, `minifb`, `eframe`):** compilados via executor Cargo e iniciados como processos Windows independentes com **sua própria janela nativa**, mantendo o RustLab como editor/tutor sem tentar embutir OpenGL/Vulkan/DirectX dentro do WebView2.

---

### ADR-004: Áudio Pedagógico no Desktop
**Decisão:** Reproduzir cues sonoros suaves no frontend via **Web Audio API** (sintetizando a mesma assinatura acústica do `src/audio.rs`: 110 ms, 660 Hz, envelope `sin²`, ganho máximo 8% escalado por volume `0..=100`), respeitando mudo, Modo Foco e ausência de dispositivo de áudio sem erro. O `src/audio.rs` atual é preservado para a TUI Classic Linux (`#[cfg(unix)]`).

---

### ADR-005: Empacotamento Windows e WebView2
**Decisão (validada na documentação oficial do Tauri v2):**
- Gerar instalador **NSIS (`-setup.exe`)** por padrão; MSI opcional apenas se validado em pipeline dedicado.
- Configurar `webviewInstallMode: { "type": "embedBootstrapper" }` (~1,8 MB adicionais) em `tauri.conf.json`, garantindo instalação automática do WebView2 Evergreen quando ausente sem inchar o instalador em ~127 MB (`offlineInstaller`), documentando claramente a necessidade de rede apenas caso o WebView2 esteja ausente na máquina.
- Compilar o executável desktop com `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` para nunca abrir janela de console ao iniciar pelo Menu Iniciar ou atalho da Área de Trabalho.

---

## 3. Evolução Incremental para Workspace Cargo

Para **não quebrar a TUI Classic** nem fazer movimentações especulativas durante a V4.0, a refatoração seguirá extração incremental a partir da V4.1:

```text
RustLab/
  Cargo.toml                 # evoluirá para [workspace] na V4.1 mantendo o pacote raiz `rustlab`
  src/                       # TUI Classic mantida funcional + módulos progressivamente portados
  crates/
    rustlab-content/         # (V4.1/V4.2) Curriculum, v3_blueprint (213 IDs), en_tr, content/*.toml
    rustlab-core/            # (V4.2) Progress, XP 100+100, SRS, config, storage, mentor policy
    rustlab-execution/       # (V4.3) ExecutionManager Windows/Unix, Job Objects, ConPTY, LSP, Project
  apps/
    desktop/
      src-tauri/             # Shell Tauri v2, capabilities, comandos IPC tipados, NSIS config
      src/                   # Frontend React + TypeScript + Vite + CodeMirror 6 + xterm.js
  docs/                      # Documentação V3 preservada + especificações V4
```

**Regra de transição:** o pacote raiz `rustlab` continua compilando o binário TUI (`cargo run`, `cargo test --all-targets`) durante toda a migração. Primeiro isolamos `#[cfg(unix)]`/`#[cfg(windows)]` nos módulos compartilhados (V4.1-A) para só então extrair crates sem duplicar lógica.

---

## 4. Contratos IPC Tauri Tipados e Versionados

Toda comunicação frontend ↔ backend usa comandos Tauri nomeados por domínio, com DTOs `Serialize`/`Deserialize` no Rust e tipos TypeScript validados/sincronizados (ex.: via `ts-rs` ou testes de contrato em JSON schema na V4.1+). **Não existe comando genérico `run_shell` ou `exec_arbitrary`.**

### 4.1 Envelope Padrão de Erro e Cancelamento
```ts
export interface CommandError {
  code:
    | "INVALID_INPUT"
    | "PATH_FORBIDDEN"
    | "NOT_FOUND"
    | "CONSENT_REQUIRED"
    | "TOOLCHAIN_MISSING"
    | "PROCESS_BUSY"
    | "CANCELLED"
    | "TIMEOUT"
    | "IO_ERROR"
    | "INTERNAL";
  messagePtBr: string;
  detail?: string;
}
```
Operações assíncronas de longa duração retornam `taskId: string` e emitem eventos Tauri tipados (`execution://event`, `pty://data`, `lsp://event`), permitindo cancelamento determinístico pelo frontend.

### 4.2 Namespaces IPC Planejados
| Namespace | Responsabilidade | Autoridade |
|---|---|---|
| `app` / `capabilities` | Versão do app, `WindowsCapabilityReport`, tour de primeira execução, licença/sobre | Backend Rust |
| `curriculum` | Listar os 6 estágios, buscar as 213 entradas com estado de desbloqueio, carregar `Lesson` V3/clássica e glossário | Backend Rust |
| `practice` | Abrir rascunho de prática/exercício/bugfix, salvar rascunho, revelar dica (`1..4`), rodar validação Cargo real, conceder XP `100+100` | Backend Rust (única autoridade de XP) |
| `progress` | Ler resumo, SRS due cards, `error_counts`, exportar/importar bundle portátil, reset/backup | Backend Rust |
| `workspace` | Selecionar pasta via dialog nativo, abrir/criar projeto Cargo, listar árvore, ler/salvar arquivos com detecção de conflito | Backend Rust |
| `execution` | Disparar `ToolCommand` fechado (`check`, `build`, `test`, `fmt`, `clippy`, `run`, `add`, `remove`, lançamento GUI separado) e cancelar | Backend Rust |
| `pty` | Iniciar sessão ConPTY para `cargo run` interativo/jogos TUI, enviar bytes `stdin`, redimensionar (`cols`, `rows`), encerrar | Backend Rust |
| `toolchain` | Toolchain Doctor (`rustup`, `rustc`, `cargo`, `rustfmt`, `clippy`, `rust-analyzer`, alvo MSVC vs GNU, VS Build Tools, Windows SDK, WebView2) e Hello World guiado | Backend Rust |
| `libraries` | Catálogo `F9`, preview exato de diff no `Cargo.toml` + `use`, aplicar/reverter após consentimento | Backend Rust |
| `mentor` | Guia offline determinístico, consulta Ollama local, API remota opt-in com trava read-only obrigatória no Modo Aprender | Backend Rust |
| `settings` | Ler/gravar `config.toml` (tema, som, movimento, modo foco, escala de UI/editor, idioma `pt-BR`/`en-US`, IA) | Backend Rust |

---

## 5. Design Funcional, Ferris, Acessibilidade e Idioma Inicial

- **Idioma padrão:** O **RustLab Desktop V4** inicia sua experiência e onboarding em **português brasileiro (`pt-BR`)**, mantendo suporte completo ao alternador `pt-BR` / `en-US` herdado da V3. A edição TUI Classic preserva seu comportamento e suíte de testes existentes.
- **Telas principais:** Início (Retomar Aprendizado) → Mapa do Curso (6 estágios / 213 entradas) → Leitor de Lição (explicação à esquerda + demo/quiz separados) → IDE de Prática (instruções + CodeMirror 6 + saída real + dicas + XP + Ferris) → Modo Criar (explorador, abas, CodeMirror 6, ConPTY `xterm.js`, F9, mentor) → Progresso & Revisão SRS → Configurações → Ajuda/Sobre.
- **Ferris canônico:** preserva exatamente as 4 expressões (`c(o.o)c` Neutro, `c(ovo)c` Feliz, `c(o,x,o)c` Encorajador, `c(o,0)c` Confuso) com animações sutis opcionais e desativação completa via **Modo Foco** ou `prefers-reduced-motion` (`MotionMode::Off`/`Reduced`).
- **Acessibilidade e Ergonomia Windows:**
  - Layout mínimo testado em **1366×768** com painéis redimensionáveis e sem corte de controles.
  - Suporte a escalas Windows **100%, 125%, 150% e 200% DPI** + zoom independente entre interface e fonte do editor.
  - Operação integral por teclado (atalhos preservados onde fazem sentido no desktop: `F1` Dica/Hover, `F5` Executar/Testar, `F9` Bibliotecas, `Ctrl+S` Salvar, `Ctrl+K` Cancelar) com anéis de foco visíveis, rótulos ARIA para leitores de tela e temas **Claro**, **Escuro**, **Alto Contraste** e **Sistema**.
