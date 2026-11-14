//! Reprodução: a animação de abertura precisa terminar quando o motor é
//! alimentado pelo caminho de desenho real (terminal + ui::draw).

use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use rustlab::app::App;

#[test]
fn welcome_animation_finishes_through_real_draw_path() {
    let mut app = App::for_snapshot_with_animation();
    let mut terminal = Terminal::new(TestBackend::new(95, 30)).unwrap();
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(2_600) {
        terminal
            .draw(|frame| rustlab::ui::draw(frame, &mut app))
            .unwrap();
        std::thread::sleep(Duration::from_millis(40));
    }
    assert!(
        !app.anim.is_running(),
        "efeitos da abertura não terminaram em 2,6s"
    );

    // REGRESSÃO CRÍTICA (descoberta em replay de stream real): um efeito
    // dissolve solteiro sobre conteúdo estático apagava a tela inteira.
    // Um frame após o fim dos efeitos, o conteúdo original DEVE estar
    // intacto no buffer.
    terminal
        .draw(|frame| rustlab::ui::draw(frame, &mut app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let area = buf.area;
    let mut text = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            text.push_str(buf[(x, y)].symbol());
        }
    }
    assert!(
        text.contains("R U S T L A B"),
        "a abertura deixou a tela sem conteúdo"
    );
    assert!(
        text.contains("Pressione qualquer tecla"),
        "conteúdo parcial perdido"
    );
}
