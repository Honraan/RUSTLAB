# Fase 6 — projetos independentes (mini-IDE)

## Aceite do escopo

| Requisito | Entrega / evidência |
| --- | --- |
| Projetos independentes do curso | Cargo new/abrir caminho/reabrir último; recentes deduplicados (oito), caminhos canônicos e troca por Ctrl+O/N/P. Abrir não executa código. |
| Trabalhar em vários arquivos | Cargo.toml, fontes em src/tests/examples/benches e build.rs; F2 cria .rs sem sobrescrever, Ctrl+R atualiza a lista. A lista distingue seleção do arquivo aberto e rola até o selecionado. |
| Preservar trabalho | Salvamento antes de trocar arquivo/projeto, executar e sair; erro impede descarte. Leitura malsucedida não substitui o editor por vazio. Escrita com temporário exclusivo, sync e rename, preservando permissões. Testes em projects/create. |
| Dependências Cargo | F9 adiciona nome ou nome@versão, F10 remove nome. Argumentos explícitos, validados, sem shell. Mesmo gate de consentimento, saída/erros e cancelamento dos demais comandos. Cargo.toml aberto é recarregado se o editor não foi alterado durante o comando. |
| Ferramentas reais | F4 build / F5 run / F6 test / F7 check / F8 fmt. F12 percorre diagnósticos abrindo linha/coluna, sem rust-analyzer. |
| Console interativo por linhas | F5 fornece stdin via pipe; Enter envia, Ctrl+D envia EOF, Ctrl+K cancela o grupo de processos. Foco vai ao console ao executar; Shift+Tab permite retornar ao editor. Prompts stdout sem newline aparecem em tempo real. |
| Limites e ciclo de vida | 4096 bytes por linha de entrada, fila de 16, 256 KiB por stream de saída, 4000 entradas no console; leitura continua após truncamento. Execução interativa expira em 300 s. Canal novo em cada run, sem vazamento de entrada entre processos. |
| UI e segurança | Atalhos visíveis, rolagem por linhas visuais no console, overlay de consentimento limpo, avisos de rede e execução não isolada. Arquivos criados não aceitam .., absolutos ou diretórios simbólicos que saiam da raiz; descoberta não segue links de diretórios. Isso NÃO constitui sandbox. |

## Verificação reproduzível

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
python3 tests/tui_phase4.py target/release/rustlab
python3 tests/tui_phase6.py target/release/rustlab
```

`tests/project_flow.rs` usa Cargo real: prompt sem newline, resposta UTF-8,
EOF, segundo run, cancelamento e remoção de dependência local sem downloads.
Adição é coberta por validação de argumentos e UI; downloads reais dependem
da rede/registro e não são exigidos na suíte. O teste PTY isola dados/config,
abre projeto, cria/salva arquivo, verifica recentes, consentimento e stdin/EOF.
Snapshots cobrem recentes, dependências e revisão de saída longa.

## Limites explícitos

- **Não é sandbox**. Código, build.rs e dependências podem acessar arquivos,
  rede e processos com as permissões da conta. Somente use projetos confiáveis.
- O console **não é PTY nem shell**. Não emula ANSI, programas full-screen,
  sinais de controle ou modo raw. Entrada é somente por linhas, sem histórico
  nem edição avançada; colagem no console não envia nada automaticamente.
- `print!` deve chamar `stdout().flush()` para mostrar prompts sem newline.
  Fragmentos recebidos em leituras distintas podem ocupar linhas distintas
  no console. stdin é oferecido somente ao cargo run interativo; testes e
  exercícios continuam não interativos.
- Suporta pacotes Cargo simples com alvo padrão. Workspaces multi-pacote,
  seleção de binários, argumentos/env customizados e terminal PTY ficam fora
  deste escopo. Cargo 1.66+ fornece add/remove; use Rust stable atual.
- Arquivos novos: .rs em src/tests/examples/benches ou build.rs, até cinco
  subdiretórios; módulos novos precisam de `mod` no pai quando aplicável.
  Sem exclusão/renomeação automática nem inserção automática de módulos.
- Um editor por vez, com salvamento ao trocar; sem merge de edições externas.
  Alterações concorrentes no disco não são detectadas. Se o usuário editar
  durante fmt/add/remove, o buffer sujo é preservado em vez de recarregado.
- Recentes ficam em `[workspace].recent-projects` no config TOML; não são cópias
  nem backups dos projetos. Arquivos continuam sendo projetos Cargo normais.
