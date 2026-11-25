# Auditoria V4 no ponto de partida

A auditoria usou o commit `3ccbb11cbf1382cd94c2be8c2ce6091747cfa3f4`, na `main` limpa. O repositório tinha um pacote Rust `rustlab` 0.3.2 (edição 2024), com biblioteca e binário. A licença declarada e distribuída é GPL-3.0-only. O CI original executava no Ubuntu. O workflow de tags produzia somente o binário Linux da TUI.

## Componentes e fronteiras

| Componente | Evidência no checkout | Reuso no desktop |
| --- | --- | --- |
| Currículo | `src/curriculum`, 10 capítulos e 30 lições TOML, 12 exercícios TOML, blueprint V3 e traduções | `Curriculum::embedded`, registro e geradores V3, dependências entre entradas |
| Aprendizagem | `src/learn.rs` com Ferris, progresso, maestria, preparação e validação Cargo | Mesmas especificações, IDs, cálculo de XP, explicações locais e dados persistentes |
| Exercícios | `src/exercises.rs` usa harness temporário e testes Cargo reais | A biblioteca já separa criação do fixture. Catálogo independente de 12 exercícios requer integração específica |
| TUI | `app`, `main`, `event`, `ui`, `theme`, `editor` e `animation` dependem de terminal | Classic mantém apresentação e atalhos. A operação gráfica tem sistema de eventos e editor próprios |
| Execução | `execution` usa Tokio, subprocessos, limitação de saída e saída curta do compilador | Reuso dos tipos e da fila, com adaptação de controle de processos para Windows |
| Projetos | `projects` detecta manifesto, fontes e projeto plano. `create` contém o fluxo TUI | Reuso de descoberta e leitura/escrita com validação adicional na fronteira IPC |
| IDE e LSP | `editor`, `completion`, `highlight`, `lsp`, `doc_hint` | CodeMirror no desktop. Protocolos e catálogo aproveitáveis, integração do LSP pendente |
| Ferramentas de aprendizagem | `lab_tools` inclui simulações, depurador nativo opcional, checkpoints e formatos de compartilhamento | Validação e modelos portáveis. Apresentação e integração precisam de adaptação |
| IA | `mentor` tem guia local, Ollama e API compatível com OpenAI. HTTPS depende de `curl`; Learn Mode filtra respostas e ações | Explicador determinístico local reutilizado. Transporte opcional e consentimento remoto precisam de fluxo próprio no desktop |
| Áudio | `audio` gera um tom PCM e usa `paplay`, com detecção PulseAudio/PipeWire/ALSA | Desktop usa síntese local de dois tons, desligada por padrão |

Versões no lockfile original: Ratatui 0.30.2, Crossterm 0.29.0, TachyonFX 0.25.2, Serde 1.0.229, TOML 1.2.1 e Tokio 1.54.1. A dependência TUI de TachyonFX difere da série 0.11 do catálogo F9. As sete séries F9 observadas em `src/doc_hint.rs` são macroquad 0.4, bevy 0.20, minifb 0.27, eframe 0.31, tachyonfx 0.11, rand 0.9 e rodio 0.20. Elas são metadados de catálogo, sem evidência de compatibilidade MSVC individual no baseline.

Ferris aparece como sequências de texto em `learn::FerrisMood`. Os snapshots TUI são texto. O Git não continha arquivos de imagem, fontes ou áudio binário no baseline. O tom original é sintetizado por código. A imagem gráfica e o ícone desta migração foram desenhados para este repositório. A interface usa fontes do sistema e ícones Lucide (ISC). As dependências mantêm suas licenças próprias.

## Currículo observado

| Etapa | Teóricas | Práticas e marcos | IDs registrados | Mapeamento direto para lição TOML |
| --- | ---: | ---: | ---: | ---: |
| O Básico | 12 | 2 | 14 | 9 |
| O Meio | 36 | 12 | 48 | 20 |
| O Difícil | 36 | 25 | 61 | 1 |
| O Fim | 40 | 20 | 60 | 4 |
| Mini Paint | 0 | 12 | 12 | 0 |
| CaveGame | 0 | 18 | 18 | 0 |
| **Total** | **124** | **89** | **213** | **34** |

Todos os IDs estão marcados `Published` no código. Nove entradas têm mapeamento direto de exercício. As anotações de estado em `CURRICULUM_V3.md` divergem do código em entradas como `B-T03`. A integridade do catálogo usa o registro Rust e os testes de contagens/ritmo/DAG.

O gerador V3 usa aulas base para material lido, apresenta um quiz parametrizado pelo objetivo da entrada e cria exemplos/demos genéricos onde necessário. Os testes exigem os blocos esperados nos 213 IDs e um `PracticeSpec` não vazio nos 89 IDs pertinentes. Há 44 IDs distribuídos em 17 braços de testes especializados, um caso separado de entrada por linhas em `D09-P01`, e 44 práticas que recorrem a uma especificação de lição base. Entre as especializadas, várias compartilham a mesma solução e o mesmo teste. Por exemplo, `PAINT-01`–`PAINT-09` reutilizam o cálculo de coordenadas e `CAVE-01`–`CAVE-09` reutilizam testes de coordenadas de chunks. Esse inventário mede cobertura estrutural e reaproveitamento, enquanto a fidelidade entre objetivo e avaliação ainda precisa de revisão por ID.

Os testes Cargo de integração exercitam `B-P01`, uma prática com `cargo test`, `D09-P01` com stdin e cinco núcleos de projetos. O teste dos capstones monta um crate sem dependências e executa testes de lógica. Mini Paint e CaveGame têm modelos reutilizáveis, respectivamente `Canvas2D` e `VoxelWorld`. Os `main` dos modelos no baseline escrevem saída textual. Há um template Bevy 2D separado. A cobertura de cada milestone e de janelas gráficas em hardware Windows permanecia sem confirmação.

## Mini-IDE e riscos Windows no baseline

| Recurso | Estado observado |
| --- | --- |
| Editar/salvar UTF-8, múltiplos arquivos no projeto, diagnósticos | Coberto por fonte, testes e snapshots da TUI |
| Check, build, run, test, fmt, Clippy | Enum fechado, Cargo real, saída limitada e diagnósticos de formato curto |
| Stdin interativo | Canal por linhas e EOF. A execução integrada original usa pipes |
| Terminais e janelas externos | Modo Criar tem launchers específicos de SO. Windows usava comando externo via `cmd /C start` na TUI |
| Controle de processos | Importações Unix, `process_group(0)` e `killpg` impediam o build MSVC; explicitamente insuficientes para processos Windows |
| Caminhos | Descoberta Cargo original procura `cargo` sem `.exe`; TUI usa HOME/XDG e progressão chama extensão Unix ao criar arquivo |
| Proteção de arquivos | Canonicalização e gravação por temporário no core; fronteira desktop precisa tratar unidades, nomes reservados, junctions e edições externas |
| Configuração e progresso | TOML distintos. `RUSTLAB_CONFIG` pode apontar outra configuração. `ProgressData` v1 aceita antigas ausências de campos e inclui rascunhos e ledger XP |
| Projetos | Diretórios Cargo comuns. Preferências apontam projetos recentes e workspace. Código de projeto permanece nesse diretório |

## Execução da linha de base

Ambiente Linux x86-64. Os comandos inicialmente retornaram `cargo: command not found` (127). Após autorização de downloads foi instalado Rust 1.100.0 sem editar PATH persistente.

| Comando | Resultado na linha de base |
| --- | --- |
| `cargo fmt --all -- --check` | Sucesso |
| `cargo clippy --locked --all-targets -- -D warnings` | Sucesso |
| `cargo test --locked --all-targets` | 267 testes passaram, um teste de animação foi ignorado |
| CI original no SHA auditado | Build, formatação, Clippy, testes e dois smokes PTY reportados verdes pelo workflow Linux |

Depois da adaptação os mesmos testes e `cargo build --locked` passaram de novo. Os smokes `python3 tests/tui_phase4.py target/debug/rustlab` e `python3 tests/tui_phase6.py target/debug/rustlab` passaram. A checagem cruzada `cargo check --locked --target x86_64-pc-windows-msvc --all-targets` comprova tipos e disponibilidade de APIs, com testes e empacotamento nativos documentados separadamente. [CI original](https://github.com/Honraan/RUSTLAB/actions/runs/44561968172)
