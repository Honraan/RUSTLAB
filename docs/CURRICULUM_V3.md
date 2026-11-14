# CURRICULUM V3 — Blueprint Canônico (213 Entradas Obrigatórias)

Este documento descreve a estrutura oficial das **213 entradas obrigatórias** implementadas e verificadas em código por `src/curriculum/v3_blueprint.rs` (`validate_v3_blueprint`).

## Totais e Ritmo Verificados pelo Linter

| Estágio | Composição / Ritmo | Teoria | Prática / Marcos | Total |
| --- | --- | --- | --- | --- |
| **O BÁSICO** | `5T -> B-P01 -> 7T -> B-P02` | 12 | 2 | **14** |
| **O MEIO** | `12 ciclos × (T01 -> T02 -> T03 -> P01)` | 36 | 12 | **48** |
| **O DIFÍCIL** | `D-REV-001 + 12 ciclos × (T01 -> T02 -> T03 -> P01 -> P02)` | 36 | 25 | **61** |
| **O FIM (Core)** | `10 unidades × (T01 -> T02 -> T03 -> T04 -> P01 -> P02)` | 40 | 20 | **60** |
| **Mini Paint** | `PAINT-01..PAINT-12` | 0 | 12 | **12** |
| **CaveGame (Bevy)** | `CAVE-01..CAVE-18` | 0 | 18 | **18** |
| **TOTAL** | Currículo completo obrigatório | **124** | **89** | **213** |

## Catálogo Completo por Estágio e IDs Estáveis

### K. O BÁSICO (14 entradas)
- `B-T01` · O que é programar? (mapeado: `o-que-e-programar`, `published`)
- `B-T02` · Como o computador executa instruções (mapeado: `como-um-programa-executa`, `published`)
- `B-T03` · O terminal sem medo (`planned`, prereq: `B-T02`)
- `B-T04` · RustLab, Rust e Cargo (mapeado: `conhecendo-o-rust`, `published`, prereq: `B-T03`)
- `B-T05` · Arquivos e projeto Rust (`planned`, prereq: `B-T04`)
- `B-P01` · Meu primeiro Hello, world! (mapeado: `hello-world`, `published`, prereq: `B-T05`)
- `B-T06` · Lendo um programa linha por linha (mapeado: `anatomia-do-hello-world`, `published`, prereq: `B-P01`)
- `B-T07` · Valores e variáveis com let (mapeado: `variaveis`, `published`, prereq: `B-T06`)
- `B-T08` · Tipos primitivos sem complicar (`drafted`, prereq: `B-T07`)
- `B-T09` · Imutabilidade, mut e const (mapeado: `mutabilidade`, `published`, prereq: `B-T08`)
- `B-T10` · println! e interpolação (`planned`, prereq: `B-T09`)
- `B-T11` · Erros normais de iniciante (`planned`, prereq: `B-T10`)
- `B-T12` · Como experimentar e estudar (`planned`, prereq: `B-T11`)
- `B-P02` · Minha primeira variável (`drafted`, prereq: `B-T12`)

### L. O MEIO (48 entradas: M01–M12, cada um T01→T02→T03→P01)
- **M01 — Fundamentos de valores:** `M01-T01` Tipos numéricos e inferência · `M01-T02` Operadores e precedência · `M01-T03` Escopo e shadowing · `M01-P01` Cartão de perfil
- **M02 — Decisões e entrada:** `M02-T01` Expressões if/else · `M02-T02` Lendo texto do usuário · `M02-T03` Conversões de entrada com segurança · `M02-P01` Calculadora com escolhas
- **M03 — Repetições e padrões:** `M03-T01` loop, while e for · `M03-T02` ranges e break/continue · `M03-T03` match e padrões básicos · `M03-P01` Adivinhe o número
- **M04 — Funções e composição:** `M04-T01` Funções e assinaturas · `M04-T02` Expressões de retorno · `M04-T03` Separando responsabilidades · `M04-P01` Conversor de unidades
- **M05 — Ownership passo a passo:** `M05-T01` Valor movido e ownership · `M05-T02` Copy e Clone · `M05-T03` Escopo e Drop introdutório · `M05-P01` Desafio das transferências
- **M06 — Borrowing e slices:** `M06-T01` Referências compartilhadas · `M06-T02` Referências mutáveis · `M06-T03` Slices e noções de lifetime · `M06-P01` Inventário emprestado
- **M07 — Texto e Unicode:** `M07-T01` String, &str e bytes · `M07-T02` Chars, fatias e Unicode · `M07-T03` Formatação e conversão de texto · `M07-P01` Analisador de mensagens
- **M08 — Coleções:** `M08-T01` Arrays, slices e Vec · `M08-T02` HashMap e conjuntos · `M08-T03` Iteração por valor ou referência · `M08-P01` Lista de compras
- **M09 — Modelando dados:** `M09-T01` Structs e campos · `M09-T02` Métodos e impl · `M09-T03` Enums e padrões associados · `M09-P01` Catálogo de contatos
- **M10 — Falhas são dados:** `M10-T01` Option e ausência de valores · `M10-T02` Result e operador ? · `M10-T03` Sem panic para erros esperados · `M10-P01` Validador de cadastro
- **M11 — Expressividade:** `M11-T01` Iterator, map e filter · `M11-T02` Closures e captures básicos · `M11-T03` Patterns e if let · `M11-P01` Relatório de pontuações
- **M12 — Organizando um projeto:** `M12-T01` Módulos e visibilidade básica · `M12-T02` Testes introdutórios · `M12-T03` Cargo, crates e documentação · `M12-P01` Gerenciador de tarefas

### M. O DIFÍCIL (61 entradas: `D-REV-001` + D01–D12, cada um T01→T02→T03→P01→P02)
- `D-REV-001` · Revisão independente — seu primeiro programa integrado
- **D01 — Ownership aplicado:** `D01-T01` Borrowing não lexical · `D01-T02` Lifetimes em funções · `D01-T03` Iteração, movimentos e ownership · `D01-P01` Laboratório de borrow checker · `D01-P02` Refatoração sem cópias desnecessárias
- **D02 — Tipos e modelagem robusta:** `D02-T01` Enums com dados e padrões · `D02-T02` Traits Derive e conversões · `D02-T03` Tipos genéricos básicos · `D02-P01` Máquina de estados · `D02-P02` Parser tipado
- **D03 — Traits e abstrações:** `D03-T01` Trait bounds e impl · `D03-T02` Trait objects e dyn · `D03-T03` Associated types e default methods · `D03-P01` Biblioteca de formas · `D03-P02` Sistema de comandos extensível
- **D04 — Ponteiros inteligentes:** `D04-T01` Box e heap · `D04-T02` Rc/Arc e compartilhamento · `D04-T03` RefCell/Weak/Drop · `D04-P01` Árvore de elementos · `D04-P02` Cache compartilhado
- **D05 — Falhas e depuração:** `D05-T01` Erros de domínio · `D05-T02` Tratamento robusto e propagação · `D05-T03` Logs, diagnósticos e Clippy · `D05-P01` CLI resistente a falhas · `D05-P02` Caça-bugs documentada
- **D06 — Qualidade e projeto multi-crate:** `D06-T01` Estrutura de workspaces · `D06-T02` Testes de integração e docs · `D06-T03` Cargo features e versionamento · `D06-P01` Workspace com biblioteca · `D06-P02` Empacotamento reproduzível
- **D07 — Concorrência I:** `D07-T01` Threads e join · `D07-T02` Channels e message passing · `D07-T03` Escopos de threads · `D07-P01` Processador paralelo · `D07-P02` Pipeline com canais
- **D08 — Concorrência II:** `D08-T01` Mutex, Arc e shared state · `D08-T02` RwLock e contenção · `D08-T03` Send, Sync e data races · `D08-P01` Contador concorrente confiável · `D08-P02` Fila de trabalho controlada
- **D09 — Primeiros projetos de terminal:** `D09-T01` stdin, stdout e stderr · `D09-T02` Eventos de teclado e PTY · `D09-T03` Estado de UI e renderização incremental · `D09-P01` Menu terminal navegável · `D09-P02` Visualizador de arquivos
- **D10 — Pong no terminal:** `D10-T01` Coordenadas e layout · `D10-T02` Loop de atualização e tempo · `D10-T03` Input, colisão e separação de responsabilidades · `D10-P01` Faça a raquete subir e descer · `D10-P02` Pong completo
- **D11 — Snake no terminal:** `D11-T01` Grid, direção e regras · `D11-T02` Aleatoriedade com seed · `D11-T03` Crescimento, colisões e estado de jogo · `D11-P01` Snake básico jogável · `D11-P02` Snake completo e polido
- **D12 — Editor de texto no terminal:** `D12-T01` Buffer e cursor · `D12-T02` Salvar, abrir e preservar dados · `D12-T03` Undo/redo e eventos de UI · `D12-P01` Mini editor — edição · `D12-P02` Mini editor — aplicação completa

### N. O FIM Core (60 entradas: F01–F10, cada um T01→T02→T03→T04→P01→P02)
- **F01 — Programação assíncrona:** `F01-T01` O que async resolve · `F01-T02` async/await e Future · `F01-T03` Executores e Tokio · `F01-T04` Cancelamento, streams e backpressure · `F01-P01` Cliente assíncrono com timeout · `F01-P02` Worker assíncrono cancelável
- **F02 — Redes e serviços:** `F02-T01` TCP e protocolo de aplicação · `F02-T02` HTTP e JSON · `F02-T03` Cliente de API robusto · `F02-T04` Servidor e rotas · `F02-P01` Cliente HTTP documentado · `F02-P02` Serviço web mínimo
- **F03 — Tipos e traits avançadas:** `F03-T01` Generics avançados e monomorfização · `F03-T02` Associated types, GATs e HRTBs · `F03-T03` Pin, Unpin e self-reference · `F03-T04` Trait objects e padrões de API · `F03-P01` Biblioteca com API genérica · `F03-P02` Abstração com dyn e lifetime
- **F04 — Macros e metaprogramação:** `F04-T01` Macros declarativas · `F04-T02` Higiene e expansão · `F04-T03` Derive e procedural macros · `F04-T04` Quando não criar macros · `F04-P01` Macro de validação pequena · `F04-P02` Derive educacional mínimo
- **F05 — Unsafe, memória e FFI:** `F05-T01` Regras e invariantes do unsafe · `F05-T02` Ponteiros crus e layout · `F05-T03` FFI e ABI C · `F05-T04` Encapsulando unsafe · `F05-P01` Wrapper seguro de FFI · `F05-P02` Auditoria de unsafe
- **F06 — Engenharia Rust profissional:** `F06-T01` Git e colaboração · `F06-T02` Cargo avançado · `F06-T03` CI, clippy, fmt e supply chain · `F06-T04` Publicação e SemVer · `F06-P01` Biblioteca publicável · `F06-P02` Pipeline multiplataforma
- **F07 — Desempenho e medição:** `F07-T01` Complexidade e custo real · `F07-T02` Alocações e ownership eficiente · `F07-T03` Benchmarks e profiling · `F07-T04` Design orientado a dados · `F07-P01` Otimização com evidência · `F07-P02` Teste de carga e regressões
- **F08 — Persistência, segurança e validação:** `F08-T01` Serde e arquivos estruturados · `F08-T02` SQLite e transações · `F08-T03` Credenciais, privilégios e caminhos · `F08-T04` Migrações e recuperação · `F08-P01` Aplicação com SQLite · `F08-P02` Importador seguro
- **F09 — GUI e desenho 2D:** `F09-T01` Event loop e estado · `F09-T02` Canvas lógico e viewport · `F09-T03` Mouse, ferramentas e renderização · `F09-T04` Layouts responsivos e exportação · `F09-P01` Janela GUI com canvas · `F09-P02` Desenho com resize
- **F10 — Arquitetura, testes e entrega:** `F10-T01` Projetos maiores e módulos · `F10-T02` Testes robustos e propriedades · `F10-T03` Observabilidade, erros e recuperação · `F10-T04` Distribuição e suporte · `F10-P01` Refatoração de projeto real · `F10-P02` Entrega de release verificável

### O. Major Capstone 1 — MINI PAINT (12 marcos)
`PAINT-01` Definir escopo e estrutura → `PAINT-02` Janela nativa e event loop → `PAINT-03` Canvas lógico persistente → `PAINT-04` Sistema de coordenadas → `PAINT-05` Pincel e traço contínuo → `PAINT-06` Paleta, cores e ferramentas → `PAINT-07` UI responsiva → `PAINT-08` Desfazer, refazer e limpar → `PAINT-09` Salvar projeto e reabrir → `PAINT-10` Exportar imagem → `PAINT-11` Empacotar para Linux e Windows → `PAINT-12` Projeto final independente

### P. Ultimate Capstone — CAVEGAME / BEVY 0.20 (18 marcos)
`CAVE-01` Preparar Bevy e primeira janela → `CAVE-02` ECS — entidades e componentes → `CAVE-03` Recursos, sistemas e ciclo de atualização → `CAVE-04` Câmera 3D e coordenadas → `CAVE-05` O primeiro bloco 3D → `CAVE-06` Controles em primeira pessoa → `CAVE-07` Pulo, gravidade e colisão → `CAVE-08` Mundo em voxels (`24×24 chunks`) → `CAVE-09` Interação com blocos → `CAVE-10` Organização em chunks → `CAVE-11` Terreno procedural com seed → `CAVE-12` Malhas e faces visíveis → `CAVE-13` Chunk streaming e gerenciamento → `CAVE-14` Inventário criativo / seleção → `CAVE-15` Salvar, carregar e versionar mundos → `CAVE-16` Otimização e diagnóstico → `CAVE-17` Polimento e distribuição → `CAVE-18` Desafio final independente
