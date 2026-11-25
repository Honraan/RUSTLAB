# V4 Threat Model — Fronteiras de Confiança, Segurança Windows e Controles

**Base auditada:** `docs/V3_SECURITY.md`, `src/execution.rs`, `src/projects.rs`, `src/create.rs`, `src/mentor.rs`, `src/mentor/https.rs`

---

## 1. Declaração Fundamental de Segurança (Sem Falsa Sandbox)

> [!CAUTION]
> **Compilar ou executar código Rust escrito pelo estudante (ou dependências/projetos importados) NO HOST WINDOWS NÃO É UMA SANDBOX.**  
> Todo binário gerado por `cargo run` / `cargo test`, bem como `build.rs` (build scripts) e `proc-macro`s durante `cargo check` / `cargo build` / `rust-analyzer`, executa com os **privilégios da conta Windows do usuário**.

O RustLab V4 **jamais** anunciará execução local como isolada/sandboxed. Em vez disso, impõe controles explícitos de **consentimento informado**, **menor privilégio na interface**, **validação estrita de caminhos/argumentos no backend Rust** e **proteção contra injeção de comandos e conteúdo**.

---

## 2. Análise das 8 Fronteiras de Confiança

### Fronteira 1: WebView2 Renderer (React/CodeMirror/xterm) ↔ Backend Tauri (Rust)
- **Ativos:** acesso ao sistema de arquivos, execução de `cargo.exe`, sessões ConPTY, configurações, credenciais de IA.
- **Ameaças:** XSS ou script inesperado no WebView2 invocando comandos Tauri arbitrários; carregamento de URL remota dentro da janela privilegiada.
- **Controles obrigatórios:**
  - `capabilities/main.json` em Tauri v2 com princípio de menor privilégio: apenas `core:default`, `dialog` e comandos customizados explícitos; **proibido** `shell:allow-execute` genérico ou permissões amplas `fs` (`**`) em todo o disco.
  - CSP estrita em `tauri.conf.json`: `default-src 'self'`, `script-src 'self'`, proibição de scripts remotos e de navegação para origens externas dentro do WebView principal (`navigation` restrito ao origin local do app).
  - Nenhum comando genérico `exec(cmd_line: String)` na bridge IPC; comandos aceitam apenas enums/DTOs fortemente tipados (`ToolCommand` fechado).

---

### Fronteira 2: Seletor de Projeto / Editor ↔ Sistema de Arquivos Windows (NTFS)
- **Ativos:** código do usuário, arquivos do sistema operacional (`C:\Windows`, `Program Files`), diretórios fora do projeto aberto.
- **Ameaças:** *path traversal* (`..\..`), escape via *symlinks*, *directory junctions* ou *reparse points* NTFS, prefixos verbatim `\\?\`, nomes reservados Windows (`CON`, `PRN`, `AUX`, `NUL`, `COM1`, `LPT1`), sobrescrita silenciosa de edições externas.
- **Controles obrigatórios:**
  - Reaproveitar e endurecer `Project::check_path` e `create_file` (`src/projects.rs`): canonicalizar raiz e alvo, normalizar prefixo `\\?\` apenas para comparação interna segura, rejeitar `ParentDir` (`..`), rejeitar `symlink` e atributos NTFS `FILE_ATTRIBUTE_REPARSE_POINT` (junctions) ao varrer árvores ou criar arquivos.
  - Bloquear seleção de diretórios protegidos como raiz de projeto (`Program Files`, `Windows`, pasta de instalação do RustLab).
  - Separar templates canônicos (somente leitura embutidos no binário) das cópias editáveis do aluno; nunca modificar arquivos do curso.
  - Antes de salvar no editor, verificar hash/mtime para **detectar modificações externas** e solicitar resolução ao usuário (nunca sobrescrever silenciosamente).

---

### Fronteira 3: Cargo / `build.rs` / Proc-Macros ↔ Conta Windows do Usuário
- **Ativos:** sistema de arquivos do usuário, rede local/externa, variáveis de ambiente, processos filhos.
- **Ameaças:** projeto importado malicioso executando código em `build.rs` ou `proc-macro` já no `cargo check` / `rust-analyzer`; injeção de metacaracteres via `cmd.exe` ou `powershell.exe`; processos órfãos continuando em background após cancelar/fechar o app.
- **Controles obrigatórios:**
  - **Zero invocação de shell:** substituir completamente `cmd /C start` (`src/create.rs:4770`) por `Command::new(exe_path)` com array `argv` validado e flag `CREATE_NO_WINDOW` para tarefas CLI.
  - **Consentimento em dois níveis:**
    1. Consentimento inicial explícito (herdado de `ExecConsent`) explicando que execução local não é sandbox.
    2. **Aviso de confiança por projeto externo/importado:** ao abrir uma pasta Cargo externa pela primeira vez (fora das lições oficiais e projetos criados pelo próprio estudante), exigir confirmação explícita antes de habilitar `cargo check/build/run/test` ou iniciar `rust-analyzer` (pois ambos podem acionar `build.rs` e proc-macros).
  - **Encerramento determinístico:** vincular todos os processos Cargo/aluno a um **Windows Job Object** com `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, mantendo timeouts por comando e teto de `256 KiB` por stream com drenagem contínua.
  - **Catálogo `F9` seguro:** `valid_dependency` preserva alfabeto restrito de crates; mostrar diff exato do `Cargo.toml` e exigir consentimento antes de modificar ou baixar dependências.

---

### Fronteira 4: Processos Filhos ConPTY ↔ Terminal Integrado (`xterm.js`)
- **Ativos:** buffer do terminal, memória da UI, entrada de teclado.
- **Ameaças:** *flood* de saída travando a UI (OOM/CPU 100%); sequências de escape terminais maliciosas (`OSC 52` clipboard, alteração de título enganosa, sequências de controle inválidas); sessões ConPTY órfãs após fechamento da aba/janela.
- **Controles obrigatórios:**
  - Taxa/buffer limitados no canal ConPTY → WebView (backpressure e descarte/truncamento avisado acima do limite configurado).
  - Sanitização ou desativação no `xterm.js` de sequências OSC não essenciais (desabilitar acesso automático ao clipboard via `OSC 52` e links perigosos).
  - Fechamento explícito do handle `HPCON` + encerramento do Job Object associado no `Drop`/evento `CloseRequested` da janela Tauri.

---

### Fronteira 5: Links de Documentação Externos ↔ Fronteira Local
- **Ativos:** integridade do shell Windows e prevenção contra abertura de esquemas arbitrários (`file://`, `ms-msdt:`, `javascript:`, `smb://`).
- **Ameaças:** URL malformada em diagnóstico/documento/resposta abrindo protocolo local perigoso; uso legado de `rundll32 url.dll,FileProtocolHandler` (`src/create.rs:4933`).
- **Controles obrigatórios:**
  - Aposentar `rundll32` no V4 Desktop.
  - Usar plugin/opener oficial Tauri com validação estrita no backend: permitir **apenas** `https://` (e `http://127.0.0.1` apenas se explicitamente aplicável a ferramenta local documentada), rejeitando caracteres de controle/quebras de linha e aplicando allowlist para links automáticos de docs (`doc.rust-lang.org`, `docs.rs`, `rust-lang.org`, `bevy.org`, `github.com/Honraan/RUSTLAB`).

---

### Fronteira 6: Provedor de IA (Ollama / API Remota) ↔ Dados do Estudante e Tokens
- **Ativos:** código-fonte do aluno, mensagens de erro, chave de API (`OPENAI_API_KEY` / token customizado), integridade pedagógica do Modo Aprender.
- **Ameaças:** exfiltração silenciosa de código para servidor remoto; vazamento de token em `config.toml`, logs ou arquivos temporários; prompt injection vindo de comentários no código ou erros do compilador; IA entregando resposta pronta ou alterando arquivos no Modo Aprender.
- **Controles obrigatórios:**
  - **Offline por padrão (`AiProvider::Disabled`).** API remota exige `RemoteAiConsent::Granted` explícito com nome do provedor/endpoint visível.
  - **Armazenamento seguro de credencial:** no Windows, armazenar API key opcional no **Windows Credential Manager** (via crate `keyring` com backend Win32 ou variável de ambiente em sessão), **nunca** em `config.toml`, logs ou exportações de backup.
  - **Substituição de `curl --config` (`src/mentor/https.rs`):** eliminar o arquivo temporário em `%TEMP%` contendo JSON/payload e o comando externo `curl`; usar cliente HTTPS nativo Rust em memória com TLS validado.
  - **Isolamento e política read-only (`src/mentor.rs`):** manter `<dados_nao_confiaveis>...</dados_nao_confiaveis>` (`escape_untrusted`) e `sanitize_learn_mode_response` no backend Rust como barreira técnica obrigatória no Modo Aprender — a IA nunca edita arquivos, nunca roda comandos e nunca concede XP.

---

### Fronteira 7: Instalador NSIS / Atualizador ↔ Integridade de Código
- **Ativos:** máquina do estudante no momento da instalação/atualização.
- **Ameaças:** download adulterado, MITM no auto-update, execução de instalador comprometido, vazamento de chaves de assinatura no CI/repo.
- **Controles obrigatórios:**
  - Releases construídos exclusivamente em workflow CI reproduzível com publicação de `SHA256SUMS`.
  - Se o `tauri-plugin-updater` for habilitado em V4.6+, exigir verificação de assinatura Ed25519 obrigatória (`.sig`), endpoints `https://` e confirmação explícita do usuário antes de baixar/instalar.
  - Chaves privadas de assinatura (`TAURI_SIGNING_PRIVATE_KEY` / certificado Windows) armazenadas apenas em GitHub Secrets protegidos, nunca no código.
  - Divulgação transparente de avisos SmartScreen caso o binário comunitário ainda não seja assinado com certificado EV/Trusted Signing.

---

### Fronteira 8: Conteúdo Markdown, Nomes de Arquivo e Saída ANSI ↔ Renderização WebView2
- **Ativos:** DOM da interface React, integridade visual de erros e lições.
- **Ameaças:** código HTML/JS dentro de uma mensagem de erro do compilador, nome de pasta/projeto ou resposta de IA ser interpretado como HTML ativo no WebView2 (XSS local).
- **Controles obrigatórios:**
  - Proibido `dangerouslySetInnerHTML` e `eval` em todo o frontend React.
  - Renderizar lições Markdown com parser seguro sem HTML bruto habilitado; renderizar stdout/stderr/ANSI via parser determinístico que converte códigos SGR de cor em `<span>` React escapados pelo próprio JSX.
  - Conteúdo do curso permanece em TOML estático compilado no binário Rust (`include_str!`).

---

## 3. Auditoria Contínua, Logs e Privacidade

- **Logs estruturados (`%LOCALAPPDATA%\RustLab\logs\`):** registram timestamp, ação IPC, código de erro e versão do app; **nunca** registram código-fonte do aluno, prompts de IA ou tokens de API por padrão (diagnósticos exportáveis para suporte passam por redação explícita e confirmação do usuário).
- **Supply Chain:** incluir na esteira de QA V4.6 `cargo audit` (RustSec), `npm audit` (dependências frontend), revisão de `Cargo.lock` e verificação de segredos antes de publicar artefatos Windows.
