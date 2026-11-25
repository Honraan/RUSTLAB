// Oculta a janela de console em builds release no Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    rustlab_desktop_lib::run();
}
