# Fase 7 — rust-analyzer e IDE avançada

## Aceite do escopo

| Requisito | Entrega / evidência |
| --- | --- |
| Cliente LSP assíncrono (`rust-analyzer`) | `LspClient` em `src/lsp.rs` usando JSON-RPC sobre stdio com `src/json.rs` (sem dependências externas extras). Detecta `rust-analyzer` via `find_rust_analyzer()` com verificação prévia `--version` para não bloquear em proxies não instalados. |
| Índice local sempre disponível offline | `WorkspaceIndex` em `src/lsp.rs` indexa arquivos `.rs` do projeto, extrai símbolos (`fn`, `struct`, `enum`, `trait`, `type`, `const`, `mod`) e doc comments (`///`). Mantém `F1`, `F11`, busca e completar funcionando mesmo sem `rust-analyzer`. |
| Documentação e hover sob cursor (`F1`) | `CreateState::show_doc_at_cursor` abre `DocPopup` combinando resposta do `rust-analyzer`, índice do workspace e glossário offline ampliado (`src/doc_hint.rs`). |
| Navegação até definição (`F11`) | `F11` consulta LSP/índice local e salta para arquivo, linha e coluna da definição (`jump_to_location`). |
| Busca global no projeto (`Ctrl+G` / `Alt+G` / `F3`) | `PromptKind::ProjectSearch` e `search_project_text` localizam ocorrências em todo o projeto; `Alt+G` / `F3` alternam entre resultados. |
| Análise estática (`cargo clippy`) | `Ctrl+L` / `Alt+C` executam `ToolCommand::Clippy` com o mesmo gate de consentimento, limites de saída e parser de diagnósticos dos demais comandos Cargo. |
| Marcadores de diagnóstico e autocompletar | Editor exibe marcadores por linha na calha (`!` erro, `*` aviso, `·` nota) e `Completion::refresh_with_extras` incorpora símbolos do workspace e sugestões do LSP. |
| Agente IA lateral no Modo Criar | `Alt+A` / `Ctrl+T` (`Ctrl+Y` fora do editor) abre o painel direito `Agente` (`Focus::Agent` em `src/create.rs` / `draw_agent_panel` em `src/ui/create.rs`) consciente do arquivo atual, console e diagnósticos, com comandos (`/modelo`, `/corrigir`, `/testes`, `/aplicar`, `/limpar`), seletor Ollama (`Alt+O` / `Ctrl+O`) e aplicação do código sugerido via `Alt+Enter` (com undo por `Ctrl+Z`). |
| Criação de projetos em pastas customizadas | `d` na tela inicial do Modo Criar ou `Alt+D` no workspace define a pasta base padrão (`WorkspaceSettings::projects_dir`); `Novo projeto` (`Ctrl+N`) também aceita `subpasta/nome` ou `~/pasta/nome` (`resolve_new_project_target` em `src/projects.rs`) e `Tab` pré-preenche a pasta atual. |

## Verificação reproduzível

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib lsp::tests
cargo test --lib create::tests::f11_vai_para_definicao_f1_mostra_doc_e_ctrl_g_busca_no_projeto
cargo test --lib create::tests::agente_lateral_alterna_responde_e_aplica_com_undo
cargo test --lib create::tests::novo_projeto_aceita_pasta_diferente_e_define_pasta_padrao
cargo test --lib projects::tests::resolve_alvo_em_pasta_padrao_subpasta_e_caminho_absoluto
cargo test --test ui_snapshot create_agente_90x30
```

## Limites explícitos

- Se o binário `rust-analyzer` não estiver disponível no sistema, o Modo Criar informa `"índice local"` e usa o `WorkspaceIndex` + glossário interno em memória.
- Não realiza refatorações multi-arquivo automáticas (`rename`/`codeAction` em lote); a assistência foca navegação, inspeção de documentação, completamento e diagnósticos contínuos.
- O estado LSP reflete o buffer aberto via notificações incrementais (`didOpen`/`didChange`) e salvamento local, sem suporte a workspaces Cargo multi-pacote complexos.
