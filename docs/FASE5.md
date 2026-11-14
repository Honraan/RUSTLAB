# Fase 5 — gamificação e alertas sonoros

## Entregue

- Uma conclusão nova de lição concede **20 XP**; primeira aprovação de cada
  exercício concede **30 XP**. Chaves únicas `lesson:<id>` e
  `exercise:<id>` ficam no arquivo versionado de progresso, tornando a
  recompensa idempotente entre repetições e reinícios.
- Nível derivado: nível 1 no início e +1 a cada 100 XP. Não há XP por leitura,
  tentativa, dica ou falha; não há recompensa repetível por apertar teclas.
- Seis marcos determinísticos: Primeiros passos (1 lição), Cinco lições,
  Curso inicial, Primeiro desafio, Seis desafios e Prática constante (10
  conclusões combinadas). Contagens ignoram IDs fora do currículo atual.
- A tela Meu Progresso apresenta XP, nível, progresso até o próximo nível e
  estado dos marcos. Com gamificação oculta esses dados não são renderizados,
  mas o ledger e o progresso continuam intactos.
- Conclusão nova dá feedback no status e chama celebração visual estática e
  breve. Movimento reduzido encurta o efeito; animações desligadas não agendam
  efeito nem deixam o loop ativo.
- Cada ganho novo de XP solicita um tom de 660 Hz/110 ms com envelope suave
  e pico máximo de 8% da escala PCM (antes do volume do sistema). O ajuste
  0–100 controla sua amplitude. `sound.enabled=false` ou volume 0 silencia.

## Limite importante do som

Requer `paplay` (pacote `pulseaudio-utils` em distribuições Debian/Ubuntu) e
servidor PulseAudio ou compatibilidade Pulse do PipeWire. A reprodução ocorre
fora da thread da TUI, em fila limitada, sem shell nem arquivos temporários,
com timeout de dois segundos por player. Em sobrecarga, pedidos além da fila
de oito sons são descartados. Sem player/servidor segue em silêncio, sem BEL
de fallback. O volume final também depende do mixer e das caixas do usuário;
não há garantia de intensidade acústica absoluta. Testes não reproduzem sons.

## Persistência e compatibilidade

`xp_events` é campo opcional na estrutura versionada de progresso; arquivos
antigos TOML sem o campo carregam com ledger vazio. XP é somado do ledger em
vez de confiar em um contador independente. Um arquivo antigo não ganha
recompensas retroativas: apenas atividades concluídas novamente, se ainda
forem possíveis, podem gerar evento novo.

## Verificação

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Os testes cobrem ledger idempotente/reinício, nível, marcos por conteúdo atual,
ocultação, silêncio sem TTY/volume zero, duração/amplitude do tom e modos de movimento.
