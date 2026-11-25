# Contrato de capacidades do Windows

A matriz separa a implementação e as verificações por ambiente. Os workflows Windows usam Windows Server 2025 x64 hospedado. A plataforma final prevista para QA manual é Windows 11 x64.

| Capacidade | Na prévia | Evidência e limite |
| --- | --- | --- |
| Lições, quizzes, XP, pré-requisitos | Conteúdo canônico empacotado em pt-BR | 213 IDs validados, testes de quiz e ledger. Fidelidade individual pendente em exercícios que reutilizam conteúdo V3 |
| Criar/abrir/editar Cargo | Seletores nativos, editor CodeMirror, buffers múltiplos, arquivo novo e detecção de conflito | Testes de criação/edição e migração; diretórios de workspace multi-crate têm cobertura parcial |
| Compilar/executar/testar/formatar | Cargo real, console com saída, diagnósticos clicáveis do projeto, cancelamento, códigos de saída e avaliação de prática | Testes nativos de execução e testes determinísticos. Estado de aprovação separado de sucesso do processo |
| ConPTY | Terminal integrado para `cargo run` do projeto, stdin, ANSI e resize | Teste Windows de pseudoconsole, caminho acentuado e entrada. Pong/Snake/editor interativos ainda pedem sessões Windows |
| Projeto GUI nativo | Templates separados de Mini Paint e CaveGame | Mini Paint abriu no runner e a preservação de canvas passou em teste de lógica. CaveGame passou no check MSVC, com janela e GPU pendentes. O modelo inicial cobre uma região 32×32 e usa o núcleo voxel Classic |
| F9 | Catálogo, prévia de diferença de manifesto e verificação Cargo consentida | Teste da alteração TOML. O undo reverte o manifesto e preserva alterações concorrentes; Cargo pode atualizar o lockfile |
| Mentor | Diagnóstico pedagógico determinístico | IA remota e Ollama mantidos na TUI. O desktop não configura provedor |
| Importação | TOML v0/v1, backup e exportação | Testes de idempotência, conflito e rejeição de versão futura |
| NSIS | Workflow x64, SHA-256 e smoke de instalação | Instalação, primeira lição, persistência e desinstalação passaram no runner. O checklist registra o SHA e a matriz manual |

`cargo check` para um alvo no Linux comprova tipagem, com testes nativos, abertura da aplicação e janelas do aluno avaliados por fluxo próprio. Não há promessa de suporte a Windows 10, ARM64 ou GPUs específicas nesta prévia.
