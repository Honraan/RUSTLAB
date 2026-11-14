# Política de IA no Modo Aprender — Professor Seguro e Somente-Leitura

## 1. Fronteira Entre Modos (`src/mentor.rs`)

O RustLab separa estritamente dois papéis para a IA:

1. **Modo Aprender (`Screen::LessonView`, `Screen::Exercises`, `Screen::Browser`, `Screen::Progress`):**
   - A IA atua **exclusivamente como professora em português (`pt-BR`) e somente-leitura**.
   - Detectada por `is_learn_mode_context(ctx)` (quando `ctx.origin` não inicia com `"Modo Criar"`).
2. **Modo Criar (`Screen::Create` — Agente Antigravity com `Alt+A`):**
   - Mantém assistência completa para criação de apps, testes e refatorações, com aplicação explícita pelo usuário (`Alt+Enter`) e rastreamento de autoria IA vs estudante (`ai_authored_lines`).

## 2. O Que é Permitido e Proibido no Modo Aprender

### Permitido
- Explicar mensagens e códigos de erro do compilador (`E0308`, `E0382`, `E0384`, `E0499`, `E0502`, `E0597`, etc.) em português acessível.
- Apontar possíveis antipadrões, problemas de ownership/borrowing ou complexidade desnecessária com linguagem cautelosa (sem falsas garantias).
- Explicar objetivos da lição, termos de Cargo/toolchain e fazer perguntas guiadas para destravar o raciocínio do estudante.

### Proibido (garantido em prompt E em código)
- **Gerar código Rust, blocos cercados (` ```rust `), snippets que resolvam o exercício, patches ou edições de arquivo.**
- Executar comandos no terminal do aluno, revelar testes ocultos ou declarar que o código compilou/passou nos testes.
- Modificar qualquer buffer do estudante (apenas o estudante edita arquivos no Modo Aprender).

## 3. Aplicação Técnica (`sanitize_learn_mode_response`)

Prompts sozinhos não bastam caso um modelo local (Ollama) ou remoto desobedeça instruções:
- `build_prompt` injeta diretiva explícita de **Professor Somente-Leitura (proibido código)** sempre que `is_learn_mode_context(ctx)` for verdadeiro.
- Antes de exibir qualquer resposta no `MentorManager::submit` em contexto de aprendizagem, `sanitize_learn_mode_response` remove qualquer bloco cercado por ` ``` ` e o substitui por aviso pedagógico orientando o estudante a escrever a própria solução.
- Dados do código, diagnósticos e perguntas do aluno sempre viajam isolados em `<dados_nao_confiaveis>...</dados_nao_confiaveis>` via `escape_untrusted`.
