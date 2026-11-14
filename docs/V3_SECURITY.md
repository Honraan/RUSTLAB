# V3 Security & Privacy — Segurança, Execução e Privacidade

## 1. Honestidade sobre Execução Local
- **Executar código Rust escrito pelo usuário NO HOST NÃO É SANDBOX.**
- O RustLab exibe um modal explícito de consentimento (`ExecConsent::Ask` → `Granted`) antes de rodar comandos `cargo test` / `cargo run` / `cargo build` / `cargo add` pela primeira vez (`src/exercises.rs` e `src/create.rs`).
- Recusar (`Esc` / `n`) garante que nenhum processo filho é disparado.

## 2. Regras Obrigatórias de Execução (`src/execution.rs`)
- **Zero strings de shell:** todas as chamadas usam `tokio::process::Command::new("cargo")` com lista fechada de variantes `ToolCommand` e `argv` explícito.
- **Isolamento de grupo de processos:** processos filhos rodam em `process_group(0)` (`setsid`/PGID), permitindo que `Ctrl+K` (cancelamento) e timeout encerrem a árvore completa sem processos órfãos.
- **Limites de saída e tempo:** streaming capturado com limite máximo de bytes por fluxo, drenando pipe após o teto para evitar deadlock e OOM na UI.
- **Fixtures isoladas e limpas:** cada tentativa de exercício escreve em diretório temporário dedicado (`rustlab-exercise-<pid>-<id>-<nonce>`), separado dos testes de validação em `tests/rustlab_validation.rs`, e remove a pasta após conclusão/erro/cancelamento.

## 3. Proteção de Caminhos e Projetos
- `src/projects.rs` sanitiza nomes de projeto (`sanitize_project_name`), valida caminhos relativos dentro da raiz do projeto contra *path traversal* (`..`, caminhos absolutos fora do projeto) e impede sobrescrita silenciosa de `PLANO.md`.
- Exclusões de projeto e alterações em disco exigem ação explícita do usuário; não há remoção silenciosa nem upload automático de arquivos.

## 4. Segurança e Privacidade na Integração de IA (`src/mentor.rs`)
- **Offline por padrão:** `AiProvider::Disabled` usa apenas o guia local determinístico sem tráfego de rede.
- **Ollama local:** prefere servidor configurado pelo usuário (`http://127.0.0.1:11434` ou `RUSTLAB_OLLAMA_URL`) com seleção de modelos leves via `Alt+O`.
- **API Remota compatível com OpenAI:** exige consentimento explícito (`RemoteAiConsent::Granted`) antes de enviar qualquer código ou erro para fora da máquina.
- **Isolamento contra Prompt Injection:** todo código, saída de compilador e pergunta do usuário é encapsulado em `<dados_nao_confiaveis>...</dados_nao_confiaveis>` após `escape_untrusted`, com instrução de sistema proibindo executar instruções vindas desses blocos.
- **Professor Somente-Leitura no Modo Aprender:** ferramentas de edição e saída contendo blocos de código são bloqueadas no contexto de ensino (veja [`AI_LEARN_MODE_POLICY.md`](./AI_LEARN_MODE_POLICY.md)).
