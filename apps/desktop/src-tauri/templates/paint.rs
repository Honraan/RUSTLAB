mod model;

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, ScaleMode, Window, WindowOptions};
use model::Canvas2D;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const WIDTH: usize = 320;
const HEIGHT: usize = 220;
const HISTORY: usize = 20;
const TITLE: &str = "Mini Paint — 1 a 6: cores | Z/Y: histórico | C: limpar | E: exportar";
type Pixels = Vec<[u8; 3]>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut canvas = Canvas2D::novo(WIDTH, HEIGHT);
    let mut window = Window::new(
        TITLE,
        WIDTH * 2,
        HEIGHT * 2,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::Stretch,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(60);
    let mut buffer = vec![0; WIDTH * HEIGHT];
    let palette = [
        [41, 36, 35],
        [223, 69, 68],
        [241, 157, 66],
        [57, 135, 86],
        [71, 116, 195],
        [255, 255, 255],
    ];
    let keys = [
        Key::Key1,
        Key::Key2,
        Key::Key3,
        Key::Key4,
        Key::Key5,
        Key::Key6,
    ];
    let mut redo = Vec::<Pixels>::new();
    let mut previous = None;
    let mut color = palette[0];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (key, swatch) in keys.into_iter().zip(palette) {
            if window.is_key_pressed(key, KeyRepeat::No) {
                color = swatch;
                window.set_title(TITLE);
            }
        }
        if window.is_key_pressed(Key::Z, KeyRepeat::No) && !canvas.undo_stack.is_empty() {
            redo.push(canvas.pixels.clone());
            canvas.desfazer();
            previous = None;
        }
        if window.is_key_pressed(Key::Y, KeyRepeat::No)
            && let Some(pixels) = redo.pop()
        {
            checkpoint(&mut canvas);
            canvas.pixels = pixels;
            previous = None;
        }
        if window.is_key_pressed(Key::C, KeyRepeat::No) {
            checkpoint(&mut canvas);
            canvas.pixels.fill([255; 3]);
            redo.clear();
            previous = None;
        }
        if window.is_key_pressed(Key::E, KeyRepeat::No) {
            match export(&canvas) {
                Ok(path) => window.set_title(&format!("Mini Paint — exportado: {path}")),
                Err(error) => window.set_title(&format!("Mini Paint — falha ao exportar: {error}")),
            }
        }

        let (width, height) = window.get_size();
        let point = window
            .get_mouse_pos(MouseMode::Discard)
            .filter(|_| window.get_mouse_down(MouseButton::Left))
            .and_then(|(x, y)| canvas.mapear_janela_para_canvas(x, y, width as f32, height as f32))
            .map(|(x, y)| (x as i32, y as i32));
        if let Some((x, y)) = point {
            let (from_x, from_y) = previous.unwrap_or_else(|| {
                checkpoint(&mut canvas);
                redo.clear();
                (x, y)
            });
            canvas.tracar_linha(from_x, from_y, x, y, color);
        }
        previous = point;
        for (pixel, [r, g, b]) in buffer.iter_mut().zip(&canvas.pixels) {
            *pixel = (u32::from(*r) << 16) | (u32::from(*g) << 8) | u32::from(*b);
        }
        window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;
    }
    Ok(())
}

fn checkpoint(canvas: &mut Canvas2D) {
    if canvas.undo_stack.len() == HISTORY {
        canvas.undo_stack.remove(0);
    }
    canvas.salvar_undo();
}

fn export(canvas: &Canvas2D) -> Result<String, Box<dyn std::error::Error>> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let path = format!("desenho-{timestamp}.ppm");
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?
        .write_all(canvas.exportar_ppm().as_bytes())?;
    Ok(path)
}
