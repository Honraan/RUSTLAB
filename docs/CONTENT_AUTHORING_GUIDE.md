# Guia de Autoria de Conteúdo V3 (`CONTENT_AUTHORING_GUIDE.md`)

## 1. Regras Gerais (Seções J, R e W)

- **Idioma:** Português do Brasil (`pt-BR`), claro, acolhedor, alinhado à esquerda e livre de jargão não explicado.
- **Honestidade de status:** nunca marque uma lição como `Published` antes de escrever todos os blocos obrigatórios e testar seus exemplos/validadores. Use `Planned`, `Drafted`, `Tested` ou `Published` em `src/curriculum/v3_blueprint.rs`.
- **Ordem pedagógica:** nunca cobre em prática um conceito, macro, crate ou tratamento de `Result`/laço antes de tê-lo ensinado nas lições anteriores do blueprint.

## 2. Contrato de Lição Teórica

Toda lição teórica deve conter no TOML (`src/content/lesson/<id>.toml`):
1. Propósito em linguagem simples e vocabulário novo explicado.
2. Exemplos pequenos, compiláveis na edição 2024.
3. Bloco de atenção/equívocos comuns (`callout kind = "warning"` ou `"tip"`).
4. Demonstração visual conceitual (`[[blocks]] type = "demo"`) com aviso explícito de simplificação didática.
5. Ao menos um `quiz` (`[[blocks]] type = "quiz"`) com campo `why` explicando a alternativa certa e por que as demais falham.

## 3. Contrato de Lição Prática e Exercício

Todo exercício em `src/content/exercise/<id>.toml` (e futuras lições práticas V3) deve conter:
1. `id` estável em `kebab-case` (nunca renomear após publicado sem migração em `storage.rs`).
2. `statement` com entradas, saídas esperadas e casos de borda explícitos.
3. `skeleton` limpo e proporcional ao estágio (mais guiado em O BÁSICO, progressivamente menor em O MEIO e O DIFÍCIL).
4. `tests` separados do código editável pelo estudante (`tests/rustlab_validation.rs`) cobrindo múltiplos casos, bordas e texto UTF-8/pt-BR quando aplicável.
5. `hints` com **exatamente 4 níveis** graduais:
   - `[0]` Pergunta conceitual
   - `[1]` Dica direcional em linguagem natural
   - `[2]` Passo a passo conceitual (sem entregar código pronto no Modo Aprender)
   - `[3]` Explicação completa de estudo após tentativas prévias

## 4. Fluxo de Publicação
1. Atualizar/adicionar os arquivos TOML e mapear em `src/curriculum/mod.rs` + `v3_blueprint.rs`.
2. Atualizar `V3PubStatus` somente após passar `cargo test --lib curriculum` e validar soluções corretas + incorretas no `cargo test` real.
3. Atualizar `docs/CURRICULUM_V3.md` e `docs/LEARNING_OBJECTIVES_MATRIX.md` mantendo total sincronia com o linter.
