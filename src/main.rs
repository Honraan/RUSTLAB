//! RustLab — ponto de entrada.
//!
//! Gerencia o ciclo de vida do terminal: entra no modo alternativo,
//! instala um panic hook que restaura o terminal, e roda o event loop
//! principal. Nenhuma dependência de áudio ou rede é exigida aqui.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Se algo entrar em pânico dentro da TUI, devolvemos o terminal ao
    // estado normal antes de imprimir a mensagem, para não deixar a
    // sessão do usuário corrompida.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    match run() {
        Ok(code) => code,
        Err(err) => {
            // `ratatui::restore` é idempotente; seguro chamar mesmo se
            // o erro ocorreu antes da inicialização da TUI.
            restore_terminal();
            eprintln!("rustlab: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<ExitCode> {
    // Verifica se parece ser um terminal interativo antes de assumir a
    // tela — assim `rustlab` com saída redirecionada não quebra a TUI.
    use std::io::IsTerminal;
    if !io::stdout().is_terminal() {
        println!(
            "rustlab: este programa precisa de um terminal interativo.\n\
             Execute-o diretamente em um emulador de terminal (sem redirecionar a saída)."
        );
        return Ok(ExitCode::SUCCESS);
    }

    let mut terminal = ratatui::init();
    ratatui::crossterm::execute!(
        io::stdout(),
        ratatui::crossterm::event::EnableMouseCapture,
        ratatui::crossterm::event::EnableBracketedPaste
    )?;
    let result = rustlab::app::App::new().and_then(|mut app| app.run(&mut terminal));
    restore_terminal();
    result
}

fn restore_terminal() {
    let _ = ratatui::crossterm::execute!(
        io::stdout(),
        ratatui::crossterm::event::DisableMouseCapture,
        ratatui::crossterm::event::DisableBracketedPaste
    );
    ratatui::restore();
}
