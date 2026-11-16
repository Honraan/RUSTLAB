# Fase 8 — Mentor de IA opcional

## Aceite do escopo

| Requisito | Entrega / evidência |
| --- | --- |
| Acesso global e retorno ao contexto | `Ctrl+T` global ou `m` (Menu, Navegador, Lição e Progresso) abre `Screen::Mentor` em `src/ui/mentor.rs`, capturando código/erro/lição atuais via `mentor::build_context` e retornando à tela de origem (`Esc` / `Ctrl+T`). |
| Guia local 100% offline (`disabled`) | Quando a IA está desligada (padrão), `local_pedagogical_response` em `src/mentor.rs` entrega explicações determinísticas por código de erro do `rustc` (`E0308`, `E0382`, `E0384`, `E0425`, `E0502`, `E0499`, `E0597`, `E0277`) e guias contextuais por ação (1–6). |
| Provedor local Ollama (`ollama`) | Conecta via HTTP/TCP local (`ollama_url` padrão `http://127.0.0.1:11434`, `ollama_model`) sem enviar dados para fora da máquina. |
| Seletor dinâmico de modelos Ollama | `fetch_ollama_models` consulta `/api/tags` do Ollama local e disponibiliza os modelos instalados (mais sugestões leves padrão como `qwen2.5-coder:1.5b` offline e entrada manual). Acessível em `Configurações` (`Modelo Ollama (local)` + `r` para atualizar), no `Mentor` (`o`/`m`) e no `Agente` do Modo Criar (`Alt+O` / `Ctrl+O` / `/modelo`). |
| Agente lateral no Modo Criar | O painel `Agente` (`Alt+A` / `Ctrl+T`) reutiliza `MentorManager::submit_agent_chat`, conversa em múltiplos turnos sobre o projeto atual (com o mesmo isolamento `<dados_nao_confiaveis>` e consentimento explícito para API remota) e aplica blocos `rust` no editor com `Alt+Enter`. |
| Provedor remoto OpenAI-compatible (`remote`) | Usa `remote_url`, `remote_model` e chave em variável de ambiente (`RUSTLAB_AI_API_KEY` ou `OPENAI_API_KEY`, nunca gravada em `config.toml`). Exige consentimento explícito `RemoteAiConsent::Granted` (`Ask` solicita aprovação antes de qualquer envio). HTTPS usa `curl --config -` com cabeçalho passado via stdin. |
| Segurança e isolamento de prompt | Código do aluno, diagnósticos e pergunta livre são encapsulados em `<dados_nao_confiaveis>` com sanitização de tags internas, instruindo o modelo a ignorar comandos embutidos no código. |
| Separação entre mentoria e verdade técnica | Cabeçalho e rodapé da tela deixam explícito que o mentor explica conceitos e dá pistas, mas nunca substitui `rustc`, `cargo check` ou `cargo test`. |

## Verificação reproduzível

```sh
cargo test --lib mentor::tests
cargo test --test ui_snapshot mentor_90x30
cargo test --test ui_snapshot create_agente_90x30
```

`mentor::tests` cobre bloqueio da API remota sem consentimento, guia determinístico para códigos `rustc`, isolamento `<dados_nao_confiaveis>`, descoberta de modelos em `/api/tags`, extração de blocos Rust e respostas HTTP simuladas para Ollama e API OpenAI-compatible com `TcpListener`.

## Limites explícitos

- A resposta do provedor externo é exibida ao final da requisição (sem streaming token a token), com cancelamento por `Ctrl+K`.
- URLs com TLS (`https://`) no provedor remoto dependem de `curl` disponível no `PATH` (verifique com `rustlab --doctor`).
- A qualidade das explicações com `Ollama` ou API remota depende do modelo configurado pelo usuário; por isso o fluxo determinístico do curso não depende da IA.
