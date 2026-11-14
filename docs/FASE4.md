# Fase 4 — exercícios, validação e progresso

## Escopo entregue

Concluída para as nove lições do currículo inicial. As Etapas 3–10 continuam
planejadas no roadmap; encerrar esta fase não significa concluir o curso todo.

| Critério de aceite | Implementação e evidência |
| --- | --- |
| Catálogo determinístico e validado | Seis exercícios TOML: soma, dobro, média, paridade, maior e intervalo. Ordem pedagógica explícita, IDs estáveis, referências a lições validadas e exatamente quatro dicas não vazias. Testes de `curriculum`. |
| Prática ligada ao aprendizado | `e` nas lições de operadores/tipos numéricos seleciona um desafio relacionado; voltar preserva posição e quizzes. Catálogo tem prévia e sugestões de revisão. Testes de `learn` e snapshots. |
| Resposta editável | Editor multilinha, seleção, undo/redo, colagem, salvamento e retomada. Resultados são invalidados ao editar; edição é bloqueada enquanto os testes rodam. Testes de `exercises`. |
| Testes Rust reais | Fixture independente com biblioteca do aluno e suíte de integração separada. Cargo offline, testes sequenciais, sem comandos shell. `tests/cargo_run.rs` verifica starters que falham, soluções válidas e erros comuns que compilam. |
| Feedback honesto | Não executado, executando, passou, falhou, não compilou, timeout, cancelado e falha de início. Erros de sintaxe sem código E0xxx também são classificados. Saída limitada a 500 entradas e rolável por linhas visuais. |
| Dicas progressivas | `F1` revela; `F2`/`F3` revisam somente as dicas já reveladas. Quantidade persistente e limitada ao catálogo. `F4`/`Shift+F4` rolam dicas compridas. |
| Progresso persistente | Lições lidas/concluídas, exercícios concluídos, dicas usadas e código das respostas. Gravação com temporário exclusivo, sincronização e rename no mesmo diretório. Testes de `storage`, incluindo concorrência de temporários. |
| Recuperação de falhas | Falha de escrita mantém o código no editor e bloqueia saída que perderia a resposta. TOML antigo continua compatível; arquivo corrompido ganha backup antes de ser substituído; versão futura nunca é sobrescrita. |
| Meu Progresso real | Conta somente conteúdo do currículo atual e oferece acesso à prática. Sem XP ou conquistas fictícias. Testes de `ui::progress`, navegação e snapshots. |
| Consentimento e ciclo de vida | Abrir/restaurar não executa código. F5 exige consentimento. Ctrl+K cancela o grupo de processos; terminal, pipes e fixtures são limpos. `tests/exercise_flow.rs` e o teste PTY cobrem o fluxo real. |
| Interface utilizável | Atalhos contextuais com altura adaptada à largura, rolagem de enunciado (`F9`/`Shift+F9`), dicas e saída. Snapshots incluem textos longos e terminal 60×24. |

## Verificação reproduzível

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --lib animation::tests::welcome_completa_e_parar_de_rodar -- --ignored
cargo build
cargo build --release
python3 tests/tui_phase4.py target/release/rustlab
```

O teste PTY usa Python 3.11+ e biblioteca padrão no Linux. Ele isola config e
dados em um diretório temporário; verifica recusa e aceite do consentimento,
starter falho, erro de compilação, aprovação, persistência, retomada e saída
limpa. Não modifica o progresso real do usuário.

## Limites intencionais

- **Execução local não é sandbox.** Testes não são um mecanismo antifraude;
  o código roda com as permissões da conta, pode acessar arquivos/rede e deve
  ser confiável. `--offline` restringe a resolução de dependências do Cargo,
  não o acesso à rede pelo código executado.
- Histórico de conclusão e resultado atual são separados. Reabrir uma resposta
  mostra “Ainda não executado”; não afirma que ela passou novamente.
- Respostas são salvas com Ctrl+S, ao voltar à lista, antes dos testes e ao sair
  com Ctrl+C. Encerramento forçado/crash pode perder alterações não salvas.
- Progresso usa `$XDG_DATA_HOME/rustlab/progresso.toml` ou
  `~/.local/share/rustlab/progresso.toml`. Use uma instância por arquivo: o
  rename é atômico, mas não há merge de progresso entre instâncias simultâneas.
- Sem stdin interativo nesta fase. XP, níveis, conquistas e som pertencem à
  Fase 5; projetos mais complexos, rust-analyzer e currículo ampliado seguem
  o roadmap do README.
