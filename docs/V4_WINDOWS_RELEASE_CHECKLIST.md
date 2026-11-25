# Checklist do instalador e da release Windows

## Automatizável

- [ ] Windows Server 2025 hospedado: `cargo check --locked --all-targets`, integração Cargo da TUI e teste ConPTY.
- [ ] Windows: build frontend e testes desktop reais com Cargo, quiz/carregamento, F9, persistência e Unicode.
- [ ] Gerar `-setup.exe` com lockfiles, checksums e GPL-3.0-only; instalar, abrir e desinstalar com smoke.
- [ ] Gerar os projetos de aluno Mini Paint e CaveGame e compilar no Windows. Guardar evidência de execução separada do resultado de build.
- [ ] Manter o CI Linux original da TUI verde.

## Windows 11 x64 em máquina ou VM

- [ ] Instalar como usuário comum com WebView2 existente e, em outra imagem, verificar a instalação do bootstrapper.
- [ ] Verificar nome e ícone dos atalhos, ausência de janela de console ao abrir e desinstalação/reinstalação com progresso e projetos preservados.
- [ ] Abrir sem Cargo/Rust e sem rede. Ler `B-T01`, acertar quiz e reabrir para conferir XP. Testar os estados de toolkit ausente, linker/SDK ausentes e toolkit completo.
- [ ] Usar caminhos com espaços, acentos, unicode e nomes longos. Exercitar arquivos travados, rascunho após travamento, conflitos externos e progresso corrompido/versão futura.
- [ ] Executar projeto com stdin e tecla de seta, resize e cancelamento. Verificar árvore de processos sem órfãos. Executar Pong/Snake/editor interativos e registrar os binários usados.
- [ ] Mini Paint: desenhar, redimensionar várias vezes, desfazer/refazer e comparar pixels no PPM exportado.
- [ ] CaveGame: registrar Windows build, driver, GPU e renderer. Testar movimento, salto, mouse, colocação/quebra de blocos e janela separada.
- [ ] Ativar alto contraste, testar Narrator, IME, teclado, foco, som desligado, monitor secundário e DPI 100%, 125%, 150% e 200%.
- [ ] Realizar sessões beta representativas e registrar correções. Windows 10 e ARM64 exigem matriz específica antes de serem anunciados.

## Distribuição

Para artefatos de prévia, use o arquivo do workflow **Desktop Windows** ligado ao SHA verificado e compare com `Get-FileHash instalador-setup.exe -Algorithm SHA256`. O executável de prévia está sem assinatura e o SmartScreen pode exibir aviso. Documente WebView2, versão de Rust/SDK para compilar, requisitos de GPU dos exemplos e política de desinstalação. Publicação estável exige autorização para release/tag, registro de proveniência, notas de versão, revisão de licenças transitivas e validação da estratégia de assinatura. [GitHub Actions](https://github.com/Honraan/RUSTLAB/actions/workflows/desktop-windows.yml)
