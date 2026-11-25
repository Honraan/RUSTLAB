# Avisos de Licença e Componentes de Terceiros (RustLab 4.0)

O **RustLab** (tanto a edição **Windows Desktop** quanto a edição **TUI Classic**) é distribuído sob a licença **GNU General Public License v3.0 only (`GPL-3.0-only`)** — consulte o arquivo integral [LICENSE](LICENSE).

O aplicativo empacota e utiliza componentes open source compatíveis, dentre os quais:

## Dependências Rust Principais (`Cargo.toml` e `apps/desktop/src-tauri/Cargo.toml`)
- **Tauri (`tauri`, `tauri-build`, `tauri-plugin-dialog`, `tauri-plugin-opener`)** — Apache-2.0 / MIT
- **Ratatui (`ratatui`)** — MIT
- **TachyonFX (`tachyonfx`)** — MIT
- **portable-pty (`portable-pty`)** — MIT (Windows ConPTY wrapper)
- **Tokio (`tokio`)** — MIT
- **Serde & Serde JSON (`serde`, `serde_json`)** — MIT / Apache-2.0
- **TOML (`toml`)** — MIT / Apache-2.0
- **Reqwest (`reqwest`)** — MIT / Apache-2.0
- **Keyring (`keyring`)** — MIT / Apache-2.0
- **SHA-2 (`sha2`)** — MIT / Apache-2.0
- **Windows / windows-sys (`windows-sys`)** — MIT / Apache-2.0

## Dependências Frontend JavaScript/TypeScript (`apps/desktop/package.json`)
- **React & React DOM (`react`, `react-dom`)** — MIT
- **CodeMirror 6 (`@codemirror/*`)** — MIT
- **xterm.js (`@xterm/xterm`, `@xterm/addon-fit`)** — MIT
- **Lucide React (`lucide-react`)** — ISC
- **Vite (`vite`, `@vitejs/plugin-react`)** — MIT

## Runtime Externo no Windows
- **Microsoft Edge WebView2 Evergreen Runtime** — software proprietário da Microsoft distribuído separadamente (ou via bootstrapper oficial quando ausente no sistema do usuário), regido pelos termos de licença da Microsoft.
- **Rust Toolchain (`rustup`, `rustc`, `cargo`)** e **Visual Studio Build Tools / Windows SDK** — não são redistribuídos dentro do instalador do RustLab; são instalados apenas pelo usuário quando desejar compilar projetos locais.
