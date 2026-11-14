# Política de Avaliação de Exercícios, Dicas e Recompensa Sem Dica (`100 + 100 XP`)

## 1. Hierarquia de Evidência (Seção G1)

1. **Testes determinísticos** (`cargo test` / comportamento observado com múltiplas entradas e casos de borda) decidem corretude.
2. **Saída real do compilador, `rustfmt` e `cargo clippy`** fornecem diagnósticos de sintaxe, tipos e lints.
3. **Requisitos estruturais explícitos** (quando declarados no enunciado, ex.: definir struct ou usar `Result`) são checados sem penalizar soluções alternativas válidas.
4. **Checagem visual/manual** para GUI/PTY é rotulada como observação humana — nunca simulada como prova automática.
5. **IA não é autoridade de aprovação ou reprovação.** Soluções diferentes do gabarito que passam nos testes recebem pontuação integral sem penalidade de estilo.

## 2. Política de Mensagens de Erro e Dicas em 4 Níveis

- Erros do compilador (`rustc` / `cargo`) são exibidos primeiro ao estudante (e explicados gentilmente pelo Ferris `c(o,x,o)c` em português) e **NÃO** consomem dica nem retiram bônus de independência.
- As dicas seguem exatamente 4 níveis graduais (`src/curriculum/mod.rs` valida `hints.len() == 4`):
  - **Nível 1:** Pergunta conceitual.
  - **Nível 2:** Orientação direcional em linguagem natural.
  - **Nível 3:** Explicação passo a passo dos conceitos envolvidos.
  - **Nível 4:** Estudo completo deliberado após tentativa prévia (apenas quando o estudante opta explicitamente por revelar todas).

## 3. Pontuação V3 (`100 XP base + 100 XP independência`)

Implementada em `calculate_v3_practice_xp(hints_used, ai_used)` (`src/learn.rs`):

| Condição de conclusão da prática | XP Base | Bônus Independência | Total |
| --- | --- | --- | --- |
| Concluiu sem pedir dica opcional (`F1`) nem ajuda conceitual explícita de IA nessa tentativa | `100` | `+100` | **200 XP** |
| Concluiu após revelar 1+ dica opcional ou consultar ajuda de IA | `100` | `0` | **100 XP** |

- A elegibilidade ao bônus é idempotente por ID no `xp_events` (sem farming ao repetir o mesmo exercício).
- Avisos automáticos/passivos não retiram o bônus; apenas ajuda opcional solicitada explicitamente pelo estudante.
- Para preservar compatibilidade com testes de integração da geração anterior (`tests/exercise_flow.rs`), exercícios legados resolvidos com dica mantêm o evento `exercise:{id} = 30` quando já existente ou sob fluxo legado testado, enquanto resoluções independentes e práticas integradas V3 registram o bônus completo `100 + 100`.
