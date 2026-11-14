//! Camada de eventos: uma thread leitora de crossterm envia eventos por
//! mpsc e o loop principal consome com timeout, produzindo `Event::Tick`.
//!
//! Isso mantém a thread da UI 100% não bloqueante (requisito 10/16):
//! `event::read()` nunca é chamado direto no loop de draw.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ratatui::crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};

pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize,
    /// Bracketed paste do terminal (colar multilinha em uma peça).
    Paste(String),
    /// Batida do relógio — usada para redesenhar apenas quando há
    /// animação em andamento.
    Tick,
}

pub struct Events {
    rx: Receiver<CrosstermEvent>,
    _handle: JoinHandle<()>,
}

impl Events {
    /// A thread dorme em `event::poll` (com timeout) — custo de CPU
    /// desprezível, e eventos continuam chegando com latência baixa.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<CrosstermEvent>();
        let handle = thread::spawn(move || {
            loop {
                match event::poll(Duration::from_millis(80)) {
                    Ok(true) => {
                        if let Ok(ev) = event::read()
                            && tx.send(ev).is_err()
                        {
                            break; // receptor caiu; o app está encerrando
                        }
                    }
                    Ok(false) => {}
                    Err(_) => break,
                }
            }
        });
        Self {
            rx,
            _handle: handle,
        }
    }

    /// Bloqueia até `timeout`; devolve `Tick` quando o prazo vence sem
    /// evento real.
    pub fn next(&self, timeout: Duration) -> Event {
        match self.rx.recv_timeout(timeout) {
            Ok(CrosstermEvent::Key(k)) => Event::Key(k),
            Ok(CrosstermEvent::Mouse(m)) => Event::Mouse(m),
            Ok(CrosstermEvent::Resize(_, _)) => Event::Resize,
            Ok(CrosstermEvent::Paste(s)) => Event::Paste(s),
            Ok(_) => Event::Tick,
            Err(RecvTimeoutError::Timeout) => Event::Tick,
            // O canal só fecha quando `Events` morre, o que não acontece
            // durante o loop; ainda assim devolvemos Tick em vez de pânico.
            Err(RecvTimeoutError::Disconnected) => Event::Tick,
        }
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}
