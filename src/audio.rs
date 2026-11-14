//! Tom de XP sintetizado, curto e suave; reprodução assíncrona via paplay.
//! Sem BEL de fallback (o alerta do terminal pode ser alto).

use std::path::Path;

/// Não bloqueia a TUI. Sem paplay/servidor de áudio, segue em silêncio.
pub fn play_cue(volume: u8) {
    use std::io::IsTerminal;
    use std::sync::{OnceLock, mpsc};
    static QUEUE: OnceLock<Option<mpsc::SyncSender<u8>>> = OnceLock::new();
    if !should_play(volume, std::io::stdout().is_terminal()) {
        return;
    }
    let queue = QUEUE.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel(8);
        std::thread::Builder::new()
            .name("rustlab-audio".into())
            .spawn(move || {
                for volume in rx {
                    play_pcm(volume);
                }
            })
            .ok()
            .map(|_| tx)
    });
    if let Some(queue) = queue {
        let _ = queue.try_send(volume);
    }
}

fn should_play(volume: u8, terminal: bool) -> bool {
    volume > 0 && terminal
}

/// 110 ms, 660 Hz, envelope senoidal: sem estalos, pico máximo de 8%.
fn xp_pcm(volume: u8) -> Vec<u8> {
    let samples = 22_050 * 110 / 1000;
    let gain = f32::from(volume.min(100)) / 100.0 * 0.08;
    (0..samples)
        .flat_map(|i| {
            let phase = i as f32 / (samples - 1) as f32;
            let envelope = (std::f32::consts::PI * phase).sin().powi(2);
            let tone = (std::f32::consts::TAU * 660.0 * i as f32 / 22_050.0).sin();
            ((tone * envelope * gain * i16::MAX as f32) as i16).to_le_bytes()
        })
        .collect()
}

fn play_pcm(volume: u8) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let Ok(mut child) = Command::new("paplay")
        .args([
            "--raw",
            "--format=s16le",
            "--rate=22050",
            "--channels=1",
            "--volume=65536",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return;
    };
    // Menos de 5 KiB: cabe no pipe mesmo antes do player começar a ler.
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&xp_pcm(volume));
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioBackend {
    PipeWire,
    PulseAudio,
    Alsa,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioReport {
    pub backend: AudioBackend,
    pub detail: String,
}

impl AudioReport {
    pub fn headline(&self) -> &'static str {
        self.headline_for(false)
    }

    pub fn headline_for(&self, english: bool) -> &'static str {
        match (english, &self.backend) {
            (true, AudioBackend::PipeWire) => "PipeWire detected",
            (true, AudioBackend::PulseAudio) => "PulseAudio detected",
            (true, AudioBackend::Alsa) => "ALSA detected",
            (true, AudioBackend::None) => "No audio subsystem detected",
            (false, AudioBackend::PipeWire) => "PipeWire detectado",
            (false, AudioBackend::PulseAudio) => "PulseAudio detectado",
            (false, AudioBackend::Alsa) => "ALSA detectada",
            (false, AudioBackend::None) => "Nenhum subsistema de áudio detectado",
        }
    }

    pub fn available(&self) -> bool {
        self.backend != AudioBackend::None
    }
}

/// Sonda o sistema de forma passiva. Ordem de preferência segue a
/// realidade do Linux desktop atual: PipeWire > Pulse > ALSA puro.
pub fn detect() -> AudioReport {
    detect_from(&RealFs)
}

/// Indireção testável: a detecção só precisa inspecionar caminhos.
trait FsProbe {
    fn exists(&self, path: &Path) -> bool;
    fn runtime_dir(&self) -> Option<String>;
    fn has_alsa_cards(&self) -> bool;
}

struct RealFs;

impl FsProbe for RealFs {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    fn runtime_dir(&self) -> Option<String> {
        std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .filter(|s| !s.is_empty())
    }
    fn has_alsa_cards(&self) -> bool {
        // /sys/class/sound/card* indica placas ALSA reais.
        std::fs::read_dir(Path::new("/sys/class/sound"))
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy().starts_with("card"))
    }
}

fn detect_from(fs: &dyn FsProbe) -> AudioReport {
    let runtime = fs.runtime_dir().map(std::path::PathBuf::from);

    if let Some(rt) = &runtime {
        // PipeWire expõe um socket de servidor em XDG_RUNTIME_DIR.
        if fs.exists(&rt.join("pipewire-0")) {
            return AudioReport {
                backend: AudioBackend::PipeWire,
                detail: format!("socket encontrado em {}", rt.join("pipewire-0").display()),
            };
        }
        if fs.exists(&rt.join("pulse").join("native")) {
            // Pulse direto, ou PipeWire expondo compatibilidade Pulse;
            // sem abrir o dispositivo não temos como distinguir, e não
            // precisamos: paplay usa o protocolo Pulse em ambos os casos.
            return AudioReport {
                backend: AudioBackend::PulseAudio,
                detail: format!("socket em {}", rt.join("pulse/native").display()),
            };
        }
    }

    if fs.exists(Path::new("/dev/snd")) && fs.has_alsa_cards() {
        return AudioReport {
            backend: AudioBackend::Alsa,
            detail: "/dev/snd contém placas de áudio".into(),
        };
    }

    AudioReport {
        backend: AudioBackend::None,
        detail: "áudio não detectado; o RustLab continua funcionando sem som".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn alerta_respeita_mute_e_nao_emite_em_pipe() {
        assert!(!should_play(70, false));
        assert!(!should_play(0, true));
        assert!(should_play(1, true));
    }

    #[test]
    fn tom_de_xp_e_curto_baixo_e_respeita_volume() {
        let decode = |bytes: Vec<u8>| {
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| i16::from_le_bytes(*b).unsigned_abs())
                .max()
                .unwrap()
        };
        assert_eq!(xp_pcm(100).len(), 2425 * 2);
        assert_eq!(decode(xp_pcm(0)), 0);
        assert!(decode(xp_pcm(100)) <= (i16::MAX as f32 * 0.08) as u16);
        assert!(decode(xp_pcm(20)) < decode(xp_pcm(70)));
        assert_eq!(xp_pcm(100), xp_pcm(255));
        assert_eq!(&xp_pcm(100)[..2], &[0, 0]);
    }

    struct FakeFs {
        existing: HashSet<String>,
        runtime: Option<String>,
        alsa_cards: bool,
    }

    impl FsProbe for FakeFs {
        fn exists(&self, path: &Path) -> bool {
            self.existing.contains(&path.display().to_string())
        }
        fn runtime_dir(&self) -> Option<String> {
            self.runtime.clone()
        }
        fn has_alsa_cards(&self) -> bool {
            self.alsa_cards
        }
    }

    #[test]
    fn pipewire_tem_precedencia() {
        let fs = FakeFs {
            existing: HashSet::from(["/run/user/1000/pipewire-0".to_string()]),
            runtime: Some("/run/user/1000".into()),
            alsa_cards: false,
        };
        let r = detect_from(&fs);
        assert_eq!(r.backend, AudioBackend::PipeWire);
        assert!(r.available());
    }

    #[test]
    fn nada_detectado_nao_e_erro() {
        let fs = FakeFs {
            existing: HashSet::new(),
            runtime: None,
            alsa_cards: false,
        };
        let r = detect_from(&fs);
        assert_eq!(r.backend, AudioBackend::None);
        assert!(!r.available());
        // A mensagem deve tranquilizar, não alertar.
        assert!(r.detail.contains("funcionando sem som"));
    }

    #[test]
    fn alsa_com_placa() {
        let fs = FakeFs {
            existing: HashSet::from(["/dev/snd".to_string()]),
            runtime: None,
            alsa_cards: true,
        };
        let r = detect_from(&fs);
        assert_eq!(r.backend, AudioBackend::Alsa);
    }
}
