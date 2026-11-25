# ADR V4 — Shell e editor do desktop Windows

## Decisão

Tauri 2, WebView2, React, TypeScript, Vite e CodeMirror 6. O executável Windows empacota os arquivos estáticos e compila a aplicação Rust com o MSVC. O pacote desktop tem seu próprio lockfile. A dependência Rust por caminho permite compartilhar o currículo e regras existentes antes de considerar separar crates. A entrada e o workflow de distribuição da TUI Classic continuam nas localizações anteriores.

| Opção | Pontos fortes | Custos neste projeto |
| --- | --- | --- |
| Tauri 2 + React | WebView2 de sistema, HTML semântico para acessibilidade, ecossistema de editor e terminal, distribuição NSIS e IPC Rust tipado | Testar IPC, CSP, leitor de tela e versões WebView2 exige Windows. Dependências npm e Rust ficam em lockfiles distintos |
| egui/eframe | Interface e estado em Rust, renderização nativa, integração direta com APIs OS. Acessibilidade disponível via AccessKit de acordo com integração | Implementação adicional de editor multilinha com histórico, sintaxe e atalhos, terminal VT e formulários semânticos. Empacotamento continua exigindo projeto separado |
| Monaco | Forte familiaridade com VS Code, widgets de diagnósticos e navegação | Maior superfície de workers e CSP, ajustes para bundle offline. Serviço de linguagem Rust ainda exigiria conexão com rust-analyzer |
| CodeMirror 6 | Pacotes de sintaxe e edição, carregamento modular, controles de teclado, extensão de tema acessível | Navegação LSP e diagnósticos clicáveis requerem adaptador. Precisamos testar comportamento com IME e leitor de tela no WebView2 |

## Fronteiras

- Engine de conteúdo `rustlab::curriculum` e modelos `rustlab::learn` são a origem dos IDs, pré-requisitos, gabaritos, práticas e progresso. A interface recebe dados delimitados, com o gabarito fora da resposta IPC de leitura.
- Tauri fornece comandos fechados para o curso, diálogos nativos, arquivos do projeto ativo, catálogo e tarefas Cargo. A lógica Windows de subprocessos fica em Rust, incluindo Job Objects e ConPTY. A UI reage a estados distintos de compilação, execução e aprovação.
- Progresso usa o formato TOML do Classic em diretório próprio de dados, com trava de instância. Projetos são Cargo comuns em pastas escolhidas. Nenhum serviço HTTP externo serve a interface.
- React controla navegação e preferências locais, CodeMirror controla sessões de edição e xterm.js recebe o fluxo de bytes da pseudoconsole. Links vindos da saída do terminal não abrem páginas.

A primeira entrega usa nomes de comando e DTOs explícitos em `apps/desktop/src/api.ts`. Testes de domínio e smoke em instalação Windows exercitam o contrato. A futura geração automatizada de interfaces deve evitar drift de enum, campo anulável e caso de erro.

## Estratégia Windows

Alvo inicial `x86_64-pc-windows-msvc`. O instalador NSIS usa instalação no perfil do usuário e bootstrapper oficial do WebView2 quando necessário. O binário release usa subsistema Windows. O usuário só precisa do Rust, linker MSVC e SDK ao compilar o próprio projeto. Usar WebView2 existente evita empacotar um runtime fixo. O bootstrapper pode precisar de rede na instalação inicial. As lições funcionam com rede desativada após a instalação. Windows 10 fica fora da matriz declarada até testes específicos.

Com Visual Studio Build Tools (Desktop development with C++ e Windows SDK), Rust MSVC e Node 24 instalados, executar em PowerShell a partir da raiz:

```powershell
Set-Location apps/desktop
npm ci
npm run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis -- --locked
```

O artefato fica em `apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis`. O modo de desenvolvimento usa `npm run tauri -- dev` e Vite local. O empacotamento de distribuição usa recursos estáticos incorporados. [Pré-requisitos Tauri](https://v2.tauri.app/start/prerequisites/) · [Instaladores Tauri](https://v2.tauri.app/distribute/windows-installer/) · [Rust MSVC](https://rust-lang.github.io/rustup/installation/windows-msvc.html) · [Atualizações Tauri](https://v2.tauri.app/plugin/updater/)
