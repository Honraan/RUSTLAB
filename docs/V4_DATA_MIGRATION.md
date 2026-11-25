# Dados do aluno e migração

A aplicação desktop guarda `progresso.toml` ao lado de `rustlab.lock` no diretório `app_local_data_dir` do Tauri, normalmente `%LOCALAPPDATA%\io.github.honraan.rustlab`. O WebView2 armazena preferências gráficas e recuperação de edições de prática no perfil do aplicativo. A edição Classic lê XDG/HOME e a opção `RUSTLAB_CONFIG` quando aplicável. Os dados de ambas as edições podem ser transferidos pelo arquivo de progresso.

## Transferência explícita

1. Feche a edição de origem para estabilizar a última gravação. Faça uma cópia de `progresso.toml`. No Linux, o Classic usa `$XDG_DATA_HOME/rustlab/progresso.toml` quando definido e absoluto ou `~/.local/share/rustlab/progresso.toml`.
2. Em Configurações do desktop, escolha **Importar TOML** e confirme substituir os dados locais. A operação é uma substituição integral e mantém os eventos XP com suas chaves originais. Importações repetidas do mesmo arquivo não somam XP.
3. Guarde a cópia portátil e o nome do backup exibido. Em caso de erro ou escolha equivocada, importe o arquivo de backup, ou feche o desktop e restaure o backup byte a byte no caminho de dados do aplicativo.
4. Para voltar ao Classic, exporte para um arquivo novo e copie o TOML para o local de progresso do Classic depois de guardar a versão atual.

Versões antigas que omitem campos usam defaults compatíveis. Dados inválidos, arquivos de outro tipo e versões futuras são recusados no importador. Antes de substituir os dados ativos, o desktop guarda um backup exclusivo. A escrita de progresso usa temporário único, sincronização, fechamento e renomeação. Se a edição local estiver corrompida, o core mantém os bytes originais em arquivo `.corrupt-*` na próxima gravação. Versões locais futuras bloqueiam escrita até recuperação explícita. Projeto aberto e fonte do aluno nunca são executados durante a leitura/importação.

## Conteúdo e limites

O TOML leva lições lidas/concluídas, exercícios, rascunhos já salvos, dicas, eventos XP, maestria, erros recorrentes e marcos. Configuração TUI, credenciais, caminhos de projetos e preferências desktop ficam no perfil da edição. Copie os próprios projetos Cargo por seus diretórios e reabra-os no destino. Salve buffers externos antes de copiar. Rascunhos não salvos permanecem no perfil local e podem ser descartados explicitamente ao abrir a versão salva. A exportação recusa sobrescrever um destino existente.

O instalador administra a pasta do aplicativo. Progresso e projetos em pastas externas permanecem separados. Para remoção manual dos dados locais, exporte o progresso, feche o aplicativo e remova a pasta do perfil do desktop, inclusive os dados WebView2 associados conforme local no Windows. Essa operação remove preferências e arquivos de recuperação. Consulte também a matriz de desinstalação.
