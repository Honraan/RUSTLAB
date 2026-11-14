#!/usr/bin/env python3
"""Smoke test da mini-IDE em PTY, sem tocar nos projetos reais do usuário."""
from pathlib import Path
import sys
import tempfile
import tomllib

from tui_phase4 import Tui


def paste(tui, text):
    tui.send(b"\x1b[200~" + text.encode() + b"\x1b[201~")


def main():
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/release/rustlab").resolve()
    temp_base = "/tmp/opencode" if Path("/tmp/opencode").is_dir() else None
    with tempfile.TemporaryDirectory(prefix="rustlab-phase6-", dir=temp_base) as directory:
        root = Path(directory)
        (root / "config.toml").write_text('[ui]\nanimations="off"\n[sound]\nenabled=false\n')
        project = root / "projeto"
        (project / "src").mkdir(parents=True)
        (project / "Cargo.toml").write_text('[package]\nname="phase6_fixture"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
        source = '''use std::io::{self, Write};
fn main() {
    print!("Nome? "); io::stdout().flush().unwrap();
    let mut name = String::new(); io::stdin().read_line(&mut name).unwrap();
    println!("Ola {}", name.trim());
    let mut end = String::new(); assert_eq!(io::stdin().read_line(&mut end).unwrap(), 0);
    println!("EOF_OK");
}
'''
        (project / "src/main.rs").write_text(source)
        tui = Tui(binary, root)
        try:
            tui.wait(lambda: "R U S T L A B" in tui.text(), "abertura")
            tui.send(b"\r")
            tui.send(b"\x1b[B")  # Modo Criar
            tui.send(b"\r")
            tui.send(b"\x1b[B")  # abrir por caminho
            tui.send(b"\r")
            paste(tui, str(project))
            tui.send(b"\r")
            tui.wait(
                lambda: "projeto aberto" in tui.text() or "project opened" in tui.text(),
                "abrir projeto",
            )
            tui.send(b"\x1bOQ")  # F2 criar arquivo
            paste(tui, "src/util.rs")
            tui.send(b"\r")
            tui.wait(lambda: (project / "src/util.rs").is_file(), "novo arquivo")
            tui.send(b"\x01")
            paste(tui, "pub fn dobro(n:i32)->i32 { n*2 }\n")
            tui.send(b"\x13")  # salvar
            tui.wait(lambda: "pub fn dobro" in (project / "src/util.rs").read_text(), "salvar arquivo")
            tui.send(b"\x0f")  # Ctrl+O projeto
            paste(tui, str(project))
            tui.send(b"\r")
            tui.send(b"\x1b[15~")  # F5
            tui.wait(lambda: "sandbox" in tui.text().lower(), "consentimento")
            tui.send(b"\x1b")  # recusa
            mark = len(tui.raw)
            tui.send(b"\x1b[15~")
            tui.send(b"\r")
            tui.wait(lambda: "Nome?" in tui.text(mark), "prompt sem newline")
            paste(tui, "Zephyr")
            tui.send(b"\r")
            tui.send(b"\x04")  # EOF
            tui.wait(lambda: "EOF_OK" in tui.text(mark), "stdin e EOF")
            tui.wait(
                lambda: "sucesso" in tui.text(mark) or "succeeded" in tui.text(mark),
                "Cargo terminou",
            )
            tui.exit_cleanly()
            config = tomllib.loads((root / "config.toml").read_text())
            assert config["workspace"]["recent-projects"] == [str(project)]
            assert (project / "src/main.rs").read_text() == source
        finally:
            tui.close()
    print("PTY Fase 6: projeto, novo arquivo, salvar, recentes, consentimento, stdin e EOF OK")


if __name__ == "__main__":
    main()
