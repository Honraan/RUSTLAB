# Matriz de Objetivos de Aprendizagem (Rastreabilidade V3)

Rastreabilidade completa **Conceito / Habilidade → ID V3 → Lição/Exercício Atual → Validador → Capstone**.

| Competência / Conceito | ID(s) V3 no Blueprint | Conteúdo Atual Mapeado | Evidência / Validador | Aplicação nos Capstones |
| --- | --- | --- | --- | --- |
| Programa, compilador, terminal e Cargo | `B-T01`..`B-T05` | `o-que-e-programar`, `como-um-programa-executa`, `conhecendo-o-rust` | Quizzes + verificação de anatomia Cargo | Fundação para criar e rodar qualquer projeto (`PAINT-01`, `CAVE-01`) |
| Primeiro executável real e variáveis | `B-P01`, `B-T06`..`B-P02` | `hello-world` (com prática V3.1), `anatomia-do-hello-world`, `variaveis`, `mutabilidade` | `cargo run` / prática embutida V3.1 + quiz + erros de iniciante | Base de todo estado inicial e saída diagnóstica |
| Valores, operadores, decisões e entrada | `M01-*`, `M02-*`, `M03-*`, `M04-*` | `tipos-numericos`, `operadores`, `condicoes-lacos-e-funcoes`, ex.: `soma`, `dobro`, `media`, `paridade`, `intervalo` | `cargo test` determinístico + checagens de borda | Cálculos de coordenadas (`PAINT-04`), entrada e física simples |
| Ownership, borrowing e slices | `M05-*`, `M06-*`, `D01-*` | `ownership-na-pratica`, `borrowing-e-referencias`, `tempo-de-vida-introducao` | Refatoração de erros `E0382`/`E0502`/`E0499` sem clone cego | Compartilhamento seguro de canvas, chunks e queries ECS |
| Texto UTF-8, coleções, structs, enums e erros | `M07-*`..`M12-*`, `D02-*`, `D05-*` | `strings-e-fatias`, `tuplas-arrays-e-vetores`, `colecoes-e-iteracao`, `structs-e-metodos`, `enums-e-pattern-matching`, `option-e-ausencia`, `result-e-propagacao`, ex.: `contar-vogais`, `filtrar-positivos`, `primeira-palavra`, `retangulo-area`, `divisao-segura` | Testes unitários Unicode, estados exaustivos e `Result` | Ferramentas do Mini Paint (`PAINT-06`), persistência de mundos (`CAVE-15`) |
| Concorrência e apps interativos em terminal | `D07-*`..`D12-*` | Planejado V3.4 (Threads, Channels, Mutex, Menu PTY, Pong, Snake, Mini Editor) | Testes de concorrência + PTY real | Desacoplamento de tarefas e estado de UI (`PAINT-02`, `CAVE-13`) |
| Assincronia, rede, unsafe, CI, SQLite e GUI 2D | `F01-*`..`F10-*` | `rede-e-programacao-assincrona`, `cli-configuracao-e-serde`, `projeto-independente-*` | Testes de timeout/mock, invariantes de resize e release | Fundação direta para `PAINT-01..12` |
| Aplicação GUI 2D (`Mini Paint`) | `PAINT-01..PAINT-12` | — (V3.7) | Canvas lógico invariante ao resize + exportação + undo/redo | Competência GUI/estado exigida antes do `CAVE-01` |
| Jogo 3D Voxel em Bevy 0.20 (`CaveGame`) | `CAVE-01..CAVE-18` | Suporte preliminar `F9 -> bevy 0.20` no Modo Criar | Mundo `24×24` chunks testável, raycast, colisão, persistência | Projeto final autoral `CAVE-18` |
