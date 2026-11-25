# V4 Windows Capabilities — Contrato de Plataforma, Toolchain Doctor e Empacotamento

**Alvo primário de QA:** Windows 11 x64 (`x86_64-pc-windows-msvc`)  
**Suporte condicional:** Windows 10 x64 (`19045+`, sujeito a testes reais antes de liberação)

> [!NOTE]
> O host usado na auditoria V4.0 é **Windows 10 Home 64 bits (Build `19045`)**. Toda validação de empacotamento, ConPTY e janelas gráficas em Windows 11 x64 deverá ser executada em ambiente Windows 11 real/VM ou runner CI dedicado antes de declarar suporte total.

---

## 1. Matriz de Suporte de Sistema Operacional

| Plataforma | Status no V4 | Requisitos / Observações |
|---|---|---|
| **Windows 11 x64** | **Alvo primário de QA** | Microsoft Edge WebView2 já integrado ao sistema; ConPTY maduro; validação obrigatória em máquina/VM real para GUI e instalador. |
| **Windows 10 x64 (Build 19045+)** | **Suporte condicional documentado** | Exige WebView2 Evergreen Runtime (instalado via `embedBootstrapper` se ausente) e ConPTY atualizado; manter nos testes se todas as dependências e WebView2 permanecerem estáveis. |
| **Windows ARM64 / x86 (32-bit)** | **Fora de escopo inicial** | Somente considerar após validação explícita de Tauri, WebView2, toolchain e crates do curso. |
| **Linux (TUI Classic)** | **Mantido via edição TUI** | Continua funcional no terminal e validado no CI Ubuntu (`tui_phase4.py`, `tui_phase6.py`). |

---

## 2. Contrato de Capacidades Windows (`WindowsCapabilityReport`)

Conforme a Seção 12 da especificação V4, o backend expõe um relatório estruturado legível por máquina (`get_windows_capabilities`) para que a UI e a tela *Sobre / Diagnóstico* reflitam exatamente o que funciona no ambiente atual:

| Capacidade | Expectativa no RustLab Desktop Windows | Estado sem Rust Toolchain | Estado com Rust + MSVC |
|---|---|---|---|
| **Lições, quizzes, progresso e XP** | Funciona **100% offline**, independente de Cargo/internet | **Disponível** | **Disponível** |
| **Criar, abrir e editar projetos Rust** | Projetos Cargo reais em pastas escolhidas pelo usuário | **Disponível** (edição/salvamento) | **Disponível** |
| **Compilar, testar, formatar e checar** | Executa `cargo.exe`/`rustc.exe` locais após consentimento | **Indisponível** (orienta Toolchain Doctor) | **Disponível** |
| **Stdin, setas e jogos TUI (Pong/Snake/Editor)** | Terminal interativo via **Windows ConPTY** + `xterm.js` | **Indisponível** (exige Cargo) | **Disponível** (V4.3+) |
| **Programas GUI (`Mini Paint`, `eframe`, `minifb`, `macroquad`)** | Abrem em **janelas Windows separadas** do processo do aluno | **Indisponível** | **Disponível** (requer display/GPU) |
| **CaveGame (`bevy = "0.20"`)** | Compila e abre janela 3D própria em GPU compatível | **Indisponível** | **Condicional a MSVC + GPU (DX12/Vulkan)** |
| **Mentor de IA** | Opcional; modo offline determinístico sempre ativo | **Offline ativo**; Ollama/Remoto opcional | **Offline ativo**; Ollama/Remoto opcional |
| **Catálogo de bibliotecas `F9`** | Consulta offline; adicionar crates exige consentimento e cache/rede | **Consulta offline** | **Disponível** (download na 1ª vez) |
| **Importar progresso TUI antigo** | Importação versionada (`progresso.toml` / backup) com backup prévio | **Disponível** | **Disponível** |
| **Instalador Windows** | Instalador **NSIS (`-setup.exe`)** sem janela de console aberta | **Disponível** | **Disponível** |

Estrutura de contrato prevista no backend (`crates/rustlab-core` / `src-tauri`):

```rust
pub struct WindowsCapabilityReport {
    pub os_caption: String,
    pub os_build: String,
    pub webview2_version: Option<String>,
    pub offline_curriculum_ready: bool, // sempre true
    pub rustup_found: bool,
    pub rustc_version: Option<String>,
    pub cargo_version: Option<String>,
    pub default_host_triple: Option<String>, // distingue -msvc vs -gnullvm
    pub rustfmt_available: bool,
    pub clippy_available: bool,
    pub rust_analyzer_available: bool,
    pub msvc_linker_available: bool,
    pub windows_sdk_available: bool,
    pub can_compile_native: bool,
    pub conpty_supported: bool,
}
```

---

## 3. Empacotamento NSIS, WebView2 e Política de Atualização

### 3.1 Instalador NSIS (`-setup.exe`)
- **Artefato principal:** `RustLab_<versao>_x64-setup.exe` gerado pelo bundler oficial do Tauri v2 (`tauri build --bundles nsis`). MSI fica opcional e só será distribuído se explicitamente testado.
- **Sem console:** o binário desktop usa `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`, garantindo inicialização limpa pelo Menu Iniciar ou atalho da Área de Trabalho.
- **Modo WebView2 (`bundle.windows.webviewInstallMode`):** configurar `embedBootstrapper` (~1,8 MB no instalador). Se o sistema já tiver WebView2 Evergreen (como o Windows 11 e o host Windows 10 auditado com `158.0.4370.33`), o instalador prossegue sem baixar nada; se estiver ausente, o bootstrapper oficial da Microsoft instala o runtime.
- **Separação de responsabilidades:** instalar o RustLab Desktop **não** instala nem altera silenciosamente `rustup`, `rustc`, Visual Studio Build Tools ou a variável `PATH` do usuário.
- **Desinstalação limpa:** o desinstalador remove o app de instalação e atalhos sem apagar projetos Cargo em `Documentos\RustLab Projetos` (ou pastas escolhidas pelo usuário). A remoção opcional de cache/progresso em `%LOCALAPPDATA%\RustLab` é documentada e deixada sob escolha explícita.

### 3.2 Assinatura de Código, SmartScreen e Auto-Update
- **SmartScreen:** enquanto o projeto não possuir certificado de assinatura de código (EV / Azure Trusted Signing), builds comunitários exibirão o aviso padrão do Windows Defender SmartScreen. A página de release e a documentação informarão isso com total transparência e fornecerão checksums `SHA256SUMS`.
- **Atualizador (`tauri-plugin-updater`):** se habilitado em fases posteriores (V4.6+), será estritamente opt-in/controlado pelo usuário, validando assinatura criptográfica Ed25519 (`.sig`) sobre HTTPS, sem nunca versionar chaves privadas no repositório.

---

## 4. Assistente de Primeira Execução e Toolchain Doctor (`pt-BR`)

Ao abrir o RustLab Desktop pela primeira vez, o iniciante é recebido em **português brasileiro** com um assistente claro e não intimidador em 7 etapas:

1. **Preferências iniciais:** tema (Claro / Escuro / Sistema / Alto Contraste), nível de animação (Completas / Reduzidas / Desligadas), som (volume e mudo) e opções de acessibilidade (zoom e Modo Foco).
2. **Garantia offline:** explica que leitura de lições, quizzes, glossário e exploração do curso funcionam imediatamente **sem instalar ferramentas de compilação**.
3. **Inspeção da Toolchain Rust:** verifica em segundo plano (sem shell) `rustup`, `rustc`, `cargo`, `rustfmt`, `clippy` e `rust-analyzer`, distinguindo o target ativo (`x86_64-pc-windows-msvc` vs. `x86_64-pc-windows-gnullvm` como observado no host de auditoria).
4. **Inspeção de pré-requisitos MSVC (para target `-msvc`):** verifica presença de compilador/linker C++ (`cl.exe` / `link.exe` via `vswhere.exe` e caminhos padrão do Visual Studio) e **Windows 10/11 SDK**. Quando ausentes, exibe instruções oficiais claras (sem afirmar falsamente que C++ Build Tools são dispensáveis):
   - Carga de trabalho **"Desenvolvimento para desktop com C++"** (*Desktop development with C++*) no Visual Studio Build Tools 2022 + componente `Windows 11/10 SDK`; ou
   - Comando oficial documentado pelo `rustup` via `winget` (copiável pelo usuário, nunca executado em elevação oculta).
5. **Teste opcional de compilação ("Olá, Mundo"):** só é oferecido **após consentimento explícito**, criando um projeto temporário isolado e rodando `cargo run` real.
6. **Diagnóstico amigável de falhas:** classifica erros reais em categorias copiáveis com solução passo a passo: `PATH` incompleto (precisa reiniciar sessão/app), toolchain ausente, linker `link.exe`/SDK ausente, bloqueio de permissão/antivírus ou falha de proxy/rede.
7. **"Configurar depois":** botão sempre acessível que leva o estudante direto para as lições teóricas e quizzes offline, permitindo rodar o Doctor novamente a qualquer momento em Configurações.

> [!CAUTION]
> O Toolchain Doctor **nunca** modifica a variável `PATH` do sistema, **nunca** solicita elevação UAC silenciosa e **nunca** baixa/instala `rustup`, Visual Studio ou crates sem consentimento explícito do usuário.

---

## 5. Requisitos Windows para Catálogo `F9` e Capstones Gráficos

| Biblioteca / Capstone | Requisitos Windows estimados | Verificação real necessária (V4.4) |
|---|---|---|
| `eframe 0.31` / `minifb 0.27` (**Mini Paint**) | Toolchain MSVC/linker + driver OpenGL/WGL (ou backend compatível) | Compilar template `/paint` com janela GUI real, desenhar traço, redimensionar janela verificando preservação do `Canvas2D` e exportar PPM. |
| `bevy 0.20` (**CaveGame**) | Toolchain MSVC + Windows SDK + GPU com suporte DX12 ou Vulkan + `[profile.dev]` otimizado | Compilar projeto Bevy real no Windows, abrir janela 3D separada, validar câmera em primeira pessoa, pulo, geração de chunks por seed e colocação/quebra de blocos. |
| `macroquad 0.4` | Toolchain Windows + OpenGL | Compilar e abrir janela 2D separada respondendo a teclado/mouse. |
| `rodio 0.20` | WASAPI padrão do Windows | Testar som real quando há hardware e fallback silencioso sem dispositivo de áudio. |
| `rand 0.9` | Pure Rust (sem deps nativas) | Validar `cargo test` no Windows (`M03-P01`, `D11`, `CAVE-11`). |
| `tachyonfx 0.25` (alinhar catálogo `F9`) | Pure Rust + ConPTY para visualização terminal | Corrigir divergência `0.11` vs `0.25` no catálogo `F9` e testar renderização em ConPTY. |

---

## 6. Limites de CI vs. QA em Hardware / VM Windows Real

- **O que o CI GitHub Actions (`windows-latest`) valida:** `cargo fmt`, `cargo clippy`, `cargo test`, testes de contrato IPC, build TypeScript/Vite, geração reproduzível do instalador NSIS (`-setup.exe`) e execução de testes headless de lógica (`Canvas2D`, `VoxelWorld`, `ExerciseTest`, diagnósticos `C:\...`).
- **O que o CI headless NÃO comprova sozinho (exige VM/máquina física Windows):** renderização visual real do WebView2 em DPI 100%–200%, interação física via teclado no ConPTY com `Pong`/`Snake`/`Mini Editor`, abertura e redimensionamento visual da janela do `Mini Paint` e renderização 3D em GPU real do `CaveGame (Bevy 0.20)`. Todos os relatórios de release da V4 deverão distinguir claramente o que passou em CI do que foi validado em dispositivo Windows real.
