# RustLab 🦀

Um ambiente terminal gamificado para aprender Rust — feito para iniciantes
absolutos e sério o bastante para virar uma mini-IDE.

**Estado atual: Fases 1–9 concluídas.** O Modo Aprender reúne as **10 etapas**
do currículo em português brasileiro (**30 lições**, widgets visuais de
conceitos e **12 exercícios** com validação determinística via Cargo). O Modo
Criar oferece mini-IDE com múltiplos arquivos, console por linhas, diagnóstico
no editor, `rust-analyzer` (com fallback automático para índice local),
`cargo clippy` e plano guiado da Etapa 10 (`PLANO.md`). O Mentor opcional
(`Ctrl+T` / `m`) funciona offline por guia determinístico ou conectado a Ollama
local / API compatível com OpenAI mediante consentimento explícito.

## O que já funciona hoje

- TUI completa em Ratatui: tela de boas-vindas animada (TachyonFX), menu
  principal, navegação por teclado (setas, `j/k`, `Enter`, `Esc`, `q`,
  `Ctrl+C`), ajuda com atalhos e o estado honesto do projeto.
- **Modo Aprender funcional** (Fases 2 e 9): navegador de curso com todas as
  10 etapas (`pt-BR`, 30 lições) disponíveis, renderizador de lição com rolagem,
  progresso e realce de sintaxe Rust; quizzes com validação determinística
  (feedback imediato + "porquê" ao acertar); 7 widgets conceituais interativos
  (`Variables`, `Loop`, `Function`, `StackHeap`, `Ownership`, `Borrowing` e
  `Vector`) com aviso de simplificação didática e fallback ASCII. Conteúdo em
  pt-BR em dados TOML embutidos, validados em `cargo test`.
- **Configurações reais e persistentes** (`TOML`): tema (Cuia / Ferrugem /
  Mono acessível / Oceano), intensidade de animação (Completas / Reduzidas /
  Desligadas), visibilidade de gamificação, alertas sonoros ativos, provedor
  de IA do Mentor (`Desligado` guia local / `Ollama` / `API remota`), modelo
  local Ollama (com descoberta via `/api/tags` e tecla `r` para atualizar) e
  pasta padrão dos projetos Cargo (`workspace.projects-dir`). Salvam
  sozinhas em escrita quase-atômica; config corrompida não impede o app de
  abrir — ele avisa e usa padrões.
- Motor de animação próprio (`animation.rs`) sobre `tachyonfx`, com três
  níveis de movimento e *idle* comprovado: sem animação rodando e sem teclas,
  o app não redesenha nada (CPU ≈ 0).
- **Som de XP opcional:** cada recompensa nova toca um tom suave de 110 ms,
  com volume ajustável e reprodução assíncrona via `paplay` (PulseAudio ou
  compatibilidade PipeWire). Volume zero/desativado silencia. Sem player ou
  servidor de áudio, continua em silêncio; não usa o alerta alto do terminal.
- Detecção de UTF-8 para escolher glifos Unicode ou fallback ASCII.
- **Modo Criar (Fase 3):** abrir/criar projetos Cargo, editor multilinha com
  seleção, undo/redo, busca, auto-indentação e salvamento; `cargo run`, `test`,
  `check` e `fmt` assíncronos, console limitado e diagnósticos determinísticos.
  Execução exige consentimento e **não é sandbox** (permissões da conta do
  usuário). Ctrl+K cancela a execução. Na Fase 6, F5 aceita entrada por linhas.
- **Fase 4:** lições lidas/concluídas, exercícios concluídos e
  dicas usadas são persistidos em `~/.local/share/rustlab/progresso.toml`
  (ou `$XDG_DATA_HOME/rustlab/progresso.toml`), com escrita atômica e
  recuperação avisada de arquivo corrompido. O arquivo inválido é preservado
  como `progresso.toml.corrupt-*` antes de gravar; versões futuras bloqueiam
  salvamento para evitar perda. No navegador do curso, `e` abre
  doze exercícios em ordem pedagógica (`soma`, `dobro`, `media`, `paridade`,
  `maior`, `intervalo`, `contar-vogais`, `filtrar-positivos`,
  `primeira-palavra`, `retangulo-area`, `divisao-segura` e
  `soma-quadrados-pares`): cada um tem editor com starter, quatro dicas
  graduais e testes determinísticos executados por
  `cargo test --offline --test rustlab_validation -- --test-threads=1` num
  diretório temporário, com suíte separada do código editável. Compilação,
  testes falhos e sucesso são estados distintos.
- **Meu Progresso:** resumo real das lições lidas/concluídas, exercícios
  concluídos e dicas reveladas, com rolagem e atalho `e` para praticar.
  Respostas dos exercícios são salvas junto ao progresso com `Ctrl+S`, ao
  voltar para a lista (`Esc`), antes dos testes e ao sair com `Ctrl+C`.
  Ao reabrir um exercício, seu código é restaurado sem executar nada.
  A marca “não salvo” indica alterações pendentes; falhas de escrita são
  avisadas e impedem a saída que perderia a resposta. Editar uma resposta
  invalida seu resultado anterior, sem apagar o histórico de conclusão.
  Encerramentos forçados/crashes podem perder alterações ainda não salvas.
  Nos exercícios, `PgUp`/`PgDn` revisam as linhas da saída dos testes;
  `F2`/`F3` percorrem dicas já reveladas e `F1` revela a próxima.
  `F4`/`Shift+F4` rolam a dica; `F9`/`Shift+F9` rolam o enunciado.
  A rolagem da saída inclui quebras visuais de mensagens longas.
  Cancelamento, tempo limite e falha ao iniciar Cargo são estados distintos
  com explicação mantida no painel de resultado.
  Lições com prática relacionada exibem `e praticar`: o catálogo seleciona
  o primeiro exercício relacionado e `Esc` retorna à lição sem perder a
  posição ou as respostas dos quizzes. A lista inclui uma prévia do
   enunciado e indica as lições para revisão; praticar não conclui a lição.
- **Fase 5 — gamificação:** +20 XP na primeira conclusão de cada lição, +30
  XP no primeiro sucesso de cada exercício e marcos da Etapa 10; ledger
  persistido impede XP repetido. Nível calculado a cada 100 XP. Oito marcos
  determinísticos em Meu Progresso, além do painel de habilidades demonstradas
  para a Etapa 10. “Ocultas” remove XP/níveis/marcos da UI sem apagar
  recompensas. Celebração visual curta respeita movimento reduzido e é
  suprimida em Off.
- **Fase 6 — projetos independentes:** criar/abrir projetos Cargo (inclusive em
  pastas personalizadas com `d` / `Alt+D`, `~/pasta/nome` ou pré-preenchimento
  com `Tab`), lista dos oito recentes (`Ctrl+P`), trocar projeto (`Ctrl+O` abrir /
  `Ctrl+N` novo),
  novos arquivos Rust (`F2`) em src/tests/examples/benches e atualização da
  lista (`Ctrl+R`). Arquivos existentes não são sobrescritos pela criação;
  falhas de leitura/salvamento preservam o editor. `F4` build, `F5` run,
  `F6` test, `F7` check, `F8` fmt. `F9`/`F10` adicionam/removem dependências
  por Cargo (podem usar rede; exigem o mesmo consentimento de execução).
  `F12` percorre diagnósticos e abre arquivo/posição correspondente.
  `Shift+Tab` troca o foco. No console de F5, digite e pressione `Enter`
  para enviar uma linha; `Ctrl+D` fecha stdin (EOF), `Ctrl+K` cancela.
  Prompts sem quebra de linha são exibidos; `print!` precisa de flush no
  programa. Limite de execução interativa: 300 segundos.
  **É console por pipes, não PTY/shell/emulador de terminal**: sem programas
  full-screen, entrada raw ou suporte a sequências ANSI. Exercícios continuam
  com stdin fechado e validação determinística. Veja [docs/FASE6.md](docs/FASE6.md).
- **Fase 7 — rust-analyzer / IDE avançada:** cliente LSP assíncrono (`rust-analyzer`)
  com fallback automático para `WorkspaceIndex` offline, hover/doc no cursor (`F1`),
  ir para definição (`F11`), busca global no projeto (`Ctrl+G` e `Alt+G`/`F3`),
  `cargo clippy` (`Ctrl+L` / `Alt+C`), marcadores de diagnóstico por linha no editor
  (`!`/`*`/`·`), autocompletar enriquecido e painel lateral do **Agente IA**
  (`Alt+A` / `Ctrl+T` no workspace, com comandos `/modelo`, `/corrigir`, `/testes`,
  `/aplicar` e `Alt+Enter` para aplicar código no editor com `Ctrl+Z`). Veja
  [docs/FASE7.md](docs/FASE7.md).
- **Fase 8 — Mentor de IA opcional (`Ctrl+T` / `m`):** guia local determinístico
  offline para códigos do `rustc` quando desligado, suporte a `Ollama` local
  (com seleção dinâmica de modelos instalados via `/api/tags` por `o`/`m` no
  Mentor, `Alt+O`/`Ctrl+O` no Agente ou em Configurações) e API
  OpenAI-compatible remota mediante consentimento explícito e token por
  variável de ambiente, com isolamento `<dados_nao_confiaveis>`. Veja
  [docs/FASE8.md](docs/FASE8.md).
- **Fase 9 — currículo completo, Etapa 10 e polimento:** 30 lições nas Etapas 1–10,
  12 exercícios determinísticos Cargo, 7 widgets conceituais interativos, criação
  de `PLANO.md` (`Alt+M`), rastreamento de habilidades demonstradas na Etapa 10,
  tema `Oceano` e suíte completa de testes. Veja [docs/FASE9.md](docs/FASE9.md).
- Ciclo de vida do terminal seguro: modo alternativo, panic hook que
  restaura o terminal, recusa educada quando a saída não é um TTY.
- Arquitetura modular: `app` (estados + event loop), `ui` (telas),
  `config`, `theme`, `animation`, `audio`, `event`, `i18n` (pt-BR), editor,
  projetos e execução.
- Suíte de testes unitários, integração Cargo real e snapshots; inclui
  compilação com erro, execução, criação de projeto e cancelamento de processo.

## Rodando

Requer Rust stable (edição 2024) e um terminal interativo:

```sh
cargo run            # ou: cargo install --path .
cargo test           # unitários + snapshots de UI
python3 tests/tui_phase4.py target/release/rustlab  # após cargo build --release
python3 tests/tui_phase6.py target/release/rustlab  # smoke da mini-IDE em PTY
SNAPSHOTS_ATUALIZAR=1 cargo test --test ui_snapshot   # regravar snapshots
```

Config em `~/.config/rustlab/config.toml` (ou `RUSTLAB_CONFIG=/caminho`).

Documentação por fase: [docs/FASE4.md](docs/FASE4.md), [docs/FASE5.md](docs/FASE5.md),
[docs/FASE6.md](docs/FASE6.md), [docs/FASE7.md](docs/FASE7.md), [docs/FASE8.md](docs/FASE8.md) e
[docs/FASE9.md](docs/FASE9.md).

## Estrutura

```
src/
  main.rs      casca: ciclo de vida do terminal + panic hook
  lib.rs
  app.rs       máquina de telas, event loop, navegação (testável)
  event.rs     thread de leitura de teclado -> canal mpsc (não bloqueia a UI)
  config.rs    TOML com defaults, merge parcial e recuperação de arquivo ruim
  theme.rs     paletas semânticas (acento, sucesso, erro, bordas...)
  animation.rs wrapper do TachyonFX + MotionMode (full/reduced/off)
  audio.rs     tom suave de XP via paplay + detecção passiva de capacidade
  json.rs      parser e serializador JSON mínimo para LSP e APIs de IA
  lsp.rs       cliente rust-analyzer (JSON-RPC) + índice local de símbolos
  mentor.rs    mentor opcional (guia local determinístico, Ollama e API remota)
  i18n.rs      strings pt-BR centralizadas
  ui/          telas de aprendizagem, Modo Criar, exercícios e Mentor
  curriculum/  schema TOML + validação dos capítulos, lições e exercícios
  storage.rs   progresso do aluno, versionado e atômico
  exercises.rs editor, hints e harness Cargo temporário
  content/     capítulos, lições e exercícios TOML validados
tests/
  ui_snapshot.rs  snapshots por tela e por tamanho de terminal
  fx_finish.rs    animação termina E preserva o conteúdo (regressão real)
```

## Fases (roadmap real)

| Fase | Entrega | Status |
|------|---------|--------|
| 1 | Fundação visual, configurações, ajuda | ✅ concluída |
| 2 | Lições + navegador do curso (currículo inicial) | ✅ concluída |
| 3 | Editor + execução real com Cargo | ✅ concluída |
| 4 | Exercícios, validação, progresso (currículo inicial) | ✅ concluída |
| 5 | Alertas sonoros + gamificação (currículo inicial) | ✅ concluída |
| 6 | Projetos independentes (mini-IDE, console por linhas) | ✅ concluída |
| 7 | rust-analyzer / IDE avançada (hover, definição, busca, Clippy) | ✅ concluída |
| 8 | Mentor de IA opcional (Local / Ollama / API remota) | ✅ concluída |
| 9 | Currículo completo (Etapas 1–10), widgets visuais e polimento | ✅ concluída |

Licença: GPL 3.0
