#!/usr/bin/env python3
"""Smoke test real da TUI em PTY (Linux, apenas biblioteca padrão).

Uso: python3 tests/tui_phase4.py target/release/rustlab
Não altera progresso/configuração do usuário; executa somente o exercício embutido.
"""
import errno
import fcntl
import os
from pathlib import Path
import pty
import re
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time
import tomllib


class Tui:
    def __init__(self, binary, root):
        self.master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 36, 110, 0, 0))
        env = os.environ.copy()
        env.update(RUSTLAB_CONFIG=str(root / "config.toml"), XDG_DATA_HOME=str(root / "data"))
        self.proc = subprocess.Popen([str(binary)], stdin=slave, stdout=slave, stderr=slave, env=env)
        os.close(slave)
        self.raw = bytearray()

    def drain(self, timeout=0.05):
        ready, _, _ = select.select([self.master], [], [], timeout)
        if ready:
            try:
                self.raw.extend(os.read(self.master, 65536))
            except OSError as err:
                if err.errno != errno.EIO:
                    raise

    def wait(self, condition, label, limit=30):
        deadline = time.monotonic() + limit
        while time.monotonic() < deadline:
            self.drain()
            if condition():
                return
            if self.proc.poll() is not None:
                raise AssertionError(f"TUI encerrou durante {label}: {self.proc.returncode}")
        raise AssertionError(f"timeout: {label}\n{self.text()[-3000:]}")

    def text(self, start=0):
        return re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", bytes(self.raw[start:]).decode("utf-8", "replace"))

    def send(self, data):
        os.write(self.master, data)
        # Dá ao loop uma oportunidade de desenhar antes da próxima tecla.
        deadline = time.monotonic() + 0.1
        while time.monotonic() < deadline:
            self.drain(0.01)

    def open_soma(self):
        self.wait(lambda: "R U S T L A B" in self.text(), "boas-vindas")
        self.send(b"\r")
        self.send(b"\r")
        self.send(b"e")
        self.send(b"\r")
        self.wait(
            lambda: "Seu c" in self.text() or "Your code" in self.text(),
            "editor do exercício",
        )

    def replace(self, source):
        self.send(b"\x01")  # Ctrl+A
        self.send(b"\x1b[200~" + source.encode() + b"\x1b[201~")

    def close(self):
        if self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.proc.kill()
                self.proc.wait()
        os.close(self.master)

    def exit_cleanly(self):
        self.send(b"\x03")
        self.wait(lambda: self.proc.poll() is not None, "saída limpa", limit=10)
        assert self.proc.returncode == 0
        leftovers = list(Path(tempfile.gettempdir()).glob(f"rustlab-exercise-{self.proc.pid}-*"))
        assert not leftovers, f"fixtures não removidos: {leftovers}"


def main():
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/release/rustlab").resolve()
    temp_base = "/tmp/opencode" if Path("/tmp/opencode").is_dir() else None
    with tempfile.TemporaryDirectory(prefix="rustlab-phase4-", dir=temp_base) as directory:
        root = Path(directory)
        (root / "config.toml").write_text('[ui]\nanimations = "off"\n[sound]\nenabled = false\n')
        progress_path = root / "data/rustlab/progresso.toml"

        def progress():
            return tomllib.loads(progress_path.read_text()) if progress_path.exists() else {}

        tui = Tui(binary, root)
        try:
            tui.open_soma()
            tui.send(b"\x1b[15~")  # F5
            tui.wait(lambda: "sandbox" in tui.text(), "consentimento explícito")
            assert not progress_path.exists(), "F5 não deve executar/salvar antes do consentimento"
            tui.send(b"\x1b")  # recusa
            assert not progress_path.exists()
            tui.send(b"\x1b[15~")
            mark = len(tui.raw)
            tui.send(b"\r")  # autoriza
            tui.wait(
                lambda: "testes falharam" in tui.text(mark) or "tests failed" in tui.text(mark),
                "starter deve falhar",
            )
            assert "soma" not in progress().get("completed_exercises", [])

            tui.replace("pub fn soma(a:i32,b:i32)->i32 { a + }")
            mark = len(tui.raw)
            tui.send(b"\x1b[15~")
            tui.wait(
                lambda: "compilou" in tui.text(mark) or "did not compile" in tui.text(mark),
                "erro de compilação",
            )
            solution = "pub fn soma(a:i32,b:i32)->i32 { a + b }\n"
            tui.replace(solution)
            tui.send(b"\x1bOP")  # F1
            tui.send(b"\x1b[15~")
            tui.wait(lambda: "soma" in progress().get("completed_exercises", []), "solução aprovada e persistida")
            assert progress()["exercise_answers"]["soma"] == solution
            assert progress()["hints_used"]["soma"] == 1
            tui.exit_cleanly()
        finally:
            tui.close()

        tui = Tui(binary, root)
        try:
            tui.open_soma()
            tui.wait(
                lambda: "Ainda n" in tui.text() or "Not run yet" in tui.text(),
                "retomada sem inventar novo resultado",
            )
            assert progress()["exercise_answers"]["soma"] == solution
            assert "a + b" in tui.text(), "resposta não foi renderizada ao retomar"
            tui.exit_cleanly()
        finally:
            tui.close()
    print("PTY Fase 4: consentimento/recusa, falha, compilação, sucesso, persistência, retomada e limpeza OK")


if __name__ == "__main__":
    main()
