# Roadmap V4 Windows

| Fase | Resultado da branch de trabalho | Critério para fechar |
| --- | --- | --- |
| 4.0 Auditoria e arquitetura | Auditoria do SHA original, ADR, migração, ameaças e contrato | Baseline e testes registrados |
| 4.1 Shell e instalador | App Tauri em pt-BR, curso e NSIS no CI | Instalação limpa, menu Iniciar, janela local com primeira lição e TUI preservada |
| 4.2 Currículo e progresso | 213 IDs, aulas do registro, quiz, hints, rascunhos, importação/exportação | Validar percursos dos seis estágios com dados históricos reais e revisar práticas de fallback |
| 4.3 IDE e ConPTY | Projetos Cargo, editor, cargo check/run/test/fmt/clippy, saída, setup MSVC e ConPTY | Cobertura de cancelamento/filhos, erros de compilação, teclas e programas de terminal no Windows 11; concluir navegação por diagnóstico e LSP |
| 4.4 F9 e projetos GUI | Prévia de manifesto e verificação; projetos de partida Mini Paint e CaveGame | Exercitar as crates selecionadas e abrir janelas com GPU suportada. Expandir CaveGame a streaming, salvamento e região completa. Integrar revisões, mentor e achievements restantes |
| 4.5 Acessibilidade | Temas claro/escuro/sistema, escala, movimento, foco, áudio opcional e navegação por teclado | Validar Narrator, alto contraste, 1366×768 e DPI 100%, 125%, 150% e 200%. Observar iniciantes |
| 4.6 Hardening | Artefato CI com checksum, teste de instalação, backup, limites e privilégio de IPC reduzido | Testar locks, quedas, reparo/reinstalação, atualização assinada ou política de atualização manual, scanner de dependências e cenários WebView2 |
| 4.7 Beta fechada | Preparação técnica desta branch | Conduzir sessões com iniciantes e usuários experientes, corrigir o que for observado |
| 4.8 Release estável | Preview de branch | Validar matriz final e publicar tag/release quando autorizado |

## Aceitação da primeira fatia

- Instalar o `-setup.exe` numa VM limpa Windows x64. Confirmar atalho, licença GPL, perfil não elevado e execução sem console persistente.
- Abrir Início, Trilha, Ajuda, Configurações e a lição `B-T01`. Confirmar explicação pt-BR, quiz, Ferris e persistência após reiniciar. Repetir com rede desativada e sem Rust de estudante.
- Confirmar que a trilha permanece acessível com Cargo ausente. Instalar ferramentas de aprendizado apenas com consentimento.
- Conferir que a TUI continua rodando a regressão Linux e que os IDs originais permanecem válidos.
- Gerar instalador, licença e checksum vinculados ao commit. Desinstalar e confirmar permanência de projetos escolhidos pelo estudante.

A validação completa percorre o [checklist de release](V4_WINDOWS_RELEASE_CHECKLIST.md).
