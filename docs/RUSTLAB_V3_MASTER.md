# RUSTLAB V3 — Especificação Mestra e Referência Arquitetural

- **Versão do documento:** 3.0 / 3.1
- **Idioma voltado ao estudante:** Português do Brasil (`pt-BR`)
- **Linguagem de implementação:** Rust estável, edição 2024 (`Cargo.toml`)
- **Estrela norte pedagógica:** Aprender → compreender → escrever → compilar → executar → observar → depurar → melhorar → criar sem ajuda.

## 1. Visão Geral

O **RustLab** evolui sua base funcional atual (lições TOML embutidas, 12 exercícios Cargo determinísticos, mini-IDE completa com árvore de arquivos, console/stdin, diagnósticos, LSP `rust-analyzer`, catálogo `F9`, seletor Ollama/API remota, revisão periódica, animações TachyonFX e som opcional) para a arquitetura V3:

1. **Currículo completo em blueprint tipado e validável (`213` entradas obrigatórias):**
   - `O BÁSICO` (14 entradas: 12 teóricas + 2 práticas)
   - `O MEIO` (48 entradas: 12 ciclos × 3 teóricas + 1 prática)
   - `O DIFÍCIL` (61 entradas: `D-REV-001` + 12 ciclos × 3 teóricas + 2 práticas)
   - `O FIM` core (60 entradas: 10 unidades × 4 teóricas + 2 práticas)
   - `Mini Paint` (12 marcos de aplicação GUI 2D)
   - `CaveGame` (18 marcos em Bevy 0.20 em mundo voxel 24×24 chunks)
2. **Compromisso inegociável da V3.1:**
   - Redesign do Modo Aprender: painel superior largo e rolável para **Explicação** + painel inferior separado para **Quiz / Mini-IDE Prática** + mascote **Ferris** à direita (`c(o.o)c`, `c(ovo)c`, `c(o,x,o)c`, `c(o,0)c`).
   - Reutilização da mini-IDE existente com compilação/execução real (`F5`), saída e diagnósticos reais, validação determinística, persistência sem perda de código e recompensa `100 XP + 100 XP` (bônus sem dicas opcionais).
   - **IA somente-leitura no Modo Aprender:** explica conceitos e erros do compilador em português, sem gerar blocos de código ou resolver a tarefa pelo estudante (preservando os recursos do Agente no Modo Criar).

## 2. Regras de Honestidade Técnica e Execução

- Nunca inflar contagens com lições vazias: status de publicação explícito (`planned`, `drafted`, `tested`, `published`) em `src/curriculum/v3_blueprint.rs`.
- Nenhuma execução local é chamada de sandbox segura: exigir consentimento explícito (`ExecConsent::Granted`), `Command::new("cargo")` com `argv` estruturado (zero shell string), cancelamento (`Ctrl+K`), timeout, limite de saída e limpeza de fixtures temporários.
- Builds multiplataforma e janelas gráficas nativas (Linux/Windows, `eframe`, `minifb`, `macroquad`, `bevy 0.20`) reportam apenas o que foi realmente compilado e testado.

## 3. Documentos V3 do Repositório

- [`V3_GAP_ANALYSIS.md`](./V3_GAP_ANALYSIS.md) — Auditoria do repositório, mapa de reuso e lacunas.
- [`V3_ARCHITECTURE.md`](./V3_ARCHITECTURE.md) — Contratos de módulos, adaptador editor↔lição e migrações de schema.
- [`CURRICULUM_V3.md`](./CURRICULUM_V3.md) — Catálogo canônico completo das 213 entradas com IDs, metas e pré-requisitos.
- [`V3_ROADMAP.md`](./V3_ROADMAP.md) — Fases incrementais `V3.0` → `V3.9` com critérios de aceite vertical.
- [`V3_SECURITY.md`](./V3_SECURITY.md) — Modelo de ameaças, execução local sem shell, isolamento contra prompt injection e proteção de caminhos.
- [`LIBRARY_COMPATIBILITY.md`](./LIBRARY_COMPATIBILITY.md) & [`F9_LIBRARY_CATALOG.md`](./F9_LIBRARY_CATALOG.md) — Matriz de compatibilidade das bibliotecas `F9` e alvos Linux/Windows.
- [`LEARNING_OBJECTIVES_MATRIX.md`](./LEARNING_OBJECTIVES_MATRIX.md) — Rastreabilidade conceito → lição → prática → validador → capstone.
- [`EXERCISE_VALIDATION_POLICY.md`](./EXERCISE_VALIDATION_POLICY.md) — Hierarquia de evidência objetiva e política de dicas/XP `100+100`.
- [`AI_LEARN_MODE_POLICY.md`](./AI_LEARN_MODE_POLICY.md) — Fronteira estrita de professor somente-leitura no Modo Aprender.
- [`V3_RELEASE_GATE.md`](./V3_RELEASE_GATE.md) — Checklist de verificação automatizada, acessibilidade e testes manuais.
- [`CONTENT_AUTHORING_GUIDE.md`](./CONTENT_AUTHORING_GUIDE.md) — Guia de autoria, revisão, localização e testes de conteúdo.
