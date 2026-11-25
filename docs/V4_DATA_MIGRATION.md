# V4 Data Migration — Persistência Windows, Compatibilidade V3 e Recuperação

**Base verificada no código atual:** `src/storage.rs` (331 linhas), `src/config.rs` (778 linhas), `src/projects.rs` (524 linhas)

---

## 1. Mapa de Diretórios: V3 (Linux XDG) → V4 (Windows Desktop)

No Windows, o RustLab V4 **nunca** grava dados mutáveis do usuário sob `Program Files`, nem presume existência de `HOME` ou variáveis `XDG_*`.

| Dado | Localização atual V3 (Linux) | Localização alvo V4 (Windows) | Override / Portabilidade |
|---|---|---|---|
| **Progresso** (`progresso.toml`) | `$XDG_DATA_HOME/rustlab/progresso.toml` ou `~/.local/share/rustlab/progresso.toml` | `%LOCALAPPDATA%\RustLab\progresso.toml` | `RUSTLAB_DATA_DIR` / flag ou testes isolados |
| **Configuração** (`config.toml`) | `$RUSTLAB_CONFIG` ou `$XDG_CONFIG_HOME/rustlab/config.toml` ou `~/.config/rustlab/config.toml` | `%APPDATA%\RustLab\config.toml` (ou `%LOCALAPPDATA%\RustLab\config.toml` — manter `RUSTLAB_CONFIG` como prioridade máxima) | `RUSTLAB_CONFIG` preservado |
| **Cache compartilhado de builds Cargo** | `$XDG_CACHE_HOME/rustlab/target-cache` ou `~/.cache/rustlab/target-cache` | `%LOCALAPPDATA%\RustLab\target-cache` | Configurável em `execution.target-cache-dir` |
| **Rascunhos de recuperação (Crash Drafts)** | Memória (`lesson_drafts` / `ExerciseState::drafts`) | `%LOCALAPPDATA%\RustLab\drafts\` (autosave versionado por ID) | Limpo após salvamento/conclusão explícita |
| **Backups pré-migração e arquivos `.corrupt-*`** | Mesmo diretório de `progresso.toml` | `%LOCALAPPDATA%\RustLab\backups\` + `.corrupt-*` ao lado do arquivo | Retenção com timestamp |
| **Logs estruturados locais** | Não persistidos | `%LOCALAPPDATA%\RustLab\logs\` | Sem código-fonte nem segredos por padrão |
| **Projetos do estudante (Modo Criar)** | `~/.local/share/rustlab/projetos` | Pasta escolhida pelo usuário (padrão sugerido: `%USERPROFILE%\Documents\RustLab Projetos\`) | Projetos Cargo normais; **nunca** removidos na desinstalação |

> [!NOTE]
> Na edição **TUI Classic** compilada para Windows, `storage.rs`, `config.rs` e `projects.rs` usarão `#[cfg(windows)]` com os mesmos caminhos base (`LOCALAPPDATA` / `APPDATA` / `USERPROFILE`) para que ambas as edições no mesmo computador Windows compartilhem progresso e configurações sem colisão.

---

## 2. Evolução Compatível de `progresso.toml` (`version = 1` → `version = 2`)

### 2.1 Esquema Atual (`version = 1` em `src/storage.rs`)
Campos existentes preservados integralmente:
- `version: u32` (`1`)
- `read_lessons: BTreeSet<String>`
- `completed_lessons: BTreeSet<String>`
- `completed_exercises: BTreeSet<String>`
- `hints_used: BTreeMap<String, u8>`
- `exercise_answers: BTreeMap<String, String>`
- `xp_events: BTreeMap<String, u32>`
- `project_milestones: BTreeSet<String>`
- `srs_cards: BTreeMap<String, SrsCard>`
- `error_counts: BTreeMap<String, u32>`
- `onboarding_completed: bool`

### 2.2 Evolução Aditiva Planejada (`version = 2`)
1. Todos os campos `v1` continuam aceitos com `#[serde(default)]`.
2. Novos metadados opcionais da V4 (ex.: `last_opened_entry_id: Option<String>`, `migrated_from_v1_at: Option<String>`, `independent_completions: BTreeSet<String>`, `capstone_manual_checks: BTreeMap<String, bool>`) são adicionados com `#[serde(default)]` sem invalidar arquivos `v1`.
3. **Guarda de versão futura mantida:** se o arquivo em disco tiver `version > CURRENT_SUPPORTED_VERSION` (ex.: `version = 999`), o loader exibe aviso claro e **bloqueia sobrescrita** para preservar dados de versões mais novas.
4. **Idempotência de XP (`xp_events`):**
   - Chaves canônicas (`lesson:<id>`, `exercise:<id>`, `v3:practice:<id>:base`, `v3:practice:<id>:independent`, `project:<id>`) impedem que reabertura, reexecução ou reimportação dupliquem XP.
   - Em caso de conflito durante importação, prevalece a união de conjuntos (`completed_*`, `read_lessons`, `project_milestones`) e o máximo determinístico por chave no mapa `xp_events` (nunca soma cega).

---

## 3. Compatibilidade de `config.toml` e Padrão `pt-BR` no Desktop

1. **Campos preservados (`src/config.rs`):** `language`, `origin-language`, `[ui]` (`theme`, `animations`, `gamification`, `inlay-hints`), `[sound]` (`enabled`, `volume`), `[ai]` (`provider`, `ollama-url`, `ollama-model`, `remote-url`, `remote-model`, `remote-consent`), `[execution]` (`consent`, `shared-target-cache`, `target-cache-dir`), `[workspace]` (`last-project`, `recent-projects`, `projects-dir`).
2. **Novos campos opcionais V4 Desktop (`#[serde(default)]`):** `ui.focus_mode`, `ui.ui_scale` (`100..=150`), `ui.editor_font_size`, `ui.desktop_theme` (`system` / `light` / `dark` / `high-contrast`), `execution.first_run_doctor_completed`.
3. **Resolução da divergência de idioma padrão:**
   - Na TUI Classic (`Config::default()`), mantém-se `"en-US"` para preservar `config::tests::idioma_padrao_e_ingles_na_primeira_abertura` (`src/config.rs:762`).
   - No **RustLab Desktop V4**, quando não existe `config.toml` prévio, o inicializador desktop cria a configuração inicial com `language = "pt-BR"`, apresentando toda a experiência inicial em português brasileiro e permitindo alternância para `en-US` nas Configurações.
4. **Sanitização de caminhos Linux na importação:** ao importar um `config.toml` vindo do Linux (ex.: `workspace.last-project = "/home/aluno/..."`, `execution.target-cache-dir = "/home/..."`), o importador detecta caminhos POSIX inexistentes no Windows, descarta/arquiva os inválidos com aviso transparente e não quebra a abertura do aplicativo.

---

## 4. Exportação e Importação Portátil (Linux/Outro PC → Windows)

Uma instalação Windows **nunca** tenta varrer partições WSL ou discos externos silenciosamente. A migração entre máquinas (ou da TUI Linux para o Desktop Windows) ocorre por fluxo explícito na interface (**Configurações → Dados e Backup → Importar / Exportar**):

1. **Formatos aceitos na importação:**
   - `progresso.toml` avulso (V3/TUI Linux ou Windows);
   - Pacote portátil versionado `rustlab-backup-v1.json` contendo `{ schema_version: 1, exported_at, app_version, sha256, progress, safe_config, drafts }`.
2. **Regra de privacidade na exportação:** tokens de API nunca fazem parte do `config.toml` nem do bundle exportado; o usuário escolhe se inclui rascunhos de exercícios e preferências de IA (sem credenciais).
3. **Prévia antes de aplicar:** antes de gravar qualquer alteração, a UI mostra resumo comparativo (*lições a adicionar, exercícios/XP a mesclar, rascunhos em conflito*), permitindo **Mesclar (recomendado)** ou **Substituir após backup**.
4. **Caminhos de projetos externos:** projetos Cargo criados na TUI Linux são pastas comuns (`Cargo.toml` + `src/`). O usuário os copia normalmente e abre no Modo Criar via seletor nativo de pastas do Windows.

---

## 5. Escrita Atômica no Windows, Backup e Recuperação de Crash

### 5.1 Protocolo de Escrita Atômica Segura no Windows
No Windows, `std::fs::rename` sobre arquivo existente pode falhar transitoriamente se o Windows Defender ou indexador estiver lendo o arquivo. O backend V4 usará:
1. Criar arquivo temporário irmão único (`progresso.toml.tmp-<pid>-<seq>`) com `create_new(true)`.
2. Gravar conteúdo UTF-8 e executar `file.sync_all()`.
3. Executar substituição atômica com pequenas retentativas limitadas (backoff de poucos milissegundos apenas para `PermissionDenied` / sharing violation transitório no Windows).
4. Em caso de falha definitiva, remover o `.tmp-*` e preservar intacto o arquivo original.

### 5.2 Backup Pré-Migração e Recuperação Pós-Crash
- Antes de qualquer migração `v1 -> v2` ou importação externa, o backend grava cópia íntegra em `%LOCALAPPDATA%\RustLab\backups\progresso-<timestamp>.toml`.
- Arquivos corrompidos continuam sendo preservados como `.corrupt-<pid>-<seq>` antes de restaurar defaults, informando o usuário sem perda silenciosa.
- **Recuperação de crash:** no Modo Aprender (práticas e exercícios) e no Modo Criar, alterações não salvas são gravadas em *autosave* com *debounce* (ex.: 1–2 s) em `%LOCALAPPDATA%\RustLab\drafts\` (separado do arquivo oficial do projeto até o usuário salvar ou sem sobrescrever edições externas detectadas por hash/mtime). Na reabertura após encerramento abrupto, o RustLab oferece restauração explícita da última lição e rascunho.

---

## 6. Desinstalação, Privacidade e Matriz de Testes de Migração

- **Desinstalação limpa:** o instalador NSIS remove apenas binários/atalhos do programa; **nunca** apaga projetos do estudante. Uma caixa opcional ou instrução documentada explica onde ficam `%LOCALAPPDATA%\RustLab\` e `%APPDATA%\RustLab\` caso o usuário queira remover dados pessoais.
- **Controle de privacidade:** botão explícito em Configurações para **Exportar meus dados** e **Apagar histórico/progresso local** (com dupla confirmação e backup temporário opcional).
- **Casos obrigatórios na suíte automatizada (V4.2 / V4.6):**
  1. `progresso.toml` ausente → inicialização limpa sem aviso.
  2. `progresso.toml` `v1` mínimo (`completed_exercises = ['soma']`) → migração preserva estado e calcula XP sem duplicação.
  3. `progresso.toml` corrompido (bytes inválidos/TOML quebrado) → defaults seguros + aviso + preservação integral do `.corrupt-*`.
  4. `progresso.toml` versão futura (`version = 999`) → leitura com aviso e bloqueio estrito de sobrescrita.
  5. Reimportação repetida do mesmo backup → operação idempotente (XP total e `xp_events` idênticos).
  6. Importação de `config.toml` Linux contendo `/home/...` → caminhos POSIX inválidos descartados com aviso sem falha de inicialização.
  7. Escrita interrompida / concorrência entre múltiplas threads → arquivo válido ou estado anterior íntegro, sem vazamento de `.tmp-*`.
  8. Roundtrip de exportação → importação em instalação limpa preservando 100% das 213 entradas, rascunhos, SRS e `error_counts`.
