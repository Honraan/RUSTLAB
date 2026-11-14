//! Motor de animação — uma fina camada sobre TachyonFX.
//!
//! Decisões de arquitetura:
//! * `EffectManager` fica na thread da UI (é `!Send` por design; tudo bem,
//!   pois só é consultado dentro do `terminal.draw`).
//! * O app pergunta `is_running()` para decidir se precisa de redraw —
//!   sem efeitos rodando e sem eventos, não desenhamos nada
//!   (requisito 16: "evitar redraws desnecessários" e baixo CPU ocioso).
//! * `MotionMode` desliga ou encurta tudo: modo `Off` nunca adiciona
//!   efeitos; `Reduced` só usa fades curtos, sem movimento de células.

use std::time::Instant;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::{EffectManager, EffectTimer, Interpolation, fx};

use crate::config::MotionMode;

/// Que tipo de "momento" visual o app quer celebrar agora.
#[derive(Debug, Clone, Copy)]
pub enum ScreenFx {
    /// Boas-vindas: surge a partir da cor de fundo.
    Welcome,
    /// Entradas normais de tela: transição suave de conteúdo.
    Enter,
    /// Saída de tela (voltar ao menu): desvanecer curto.
    Leave,
    /// Celebração curta, sem deslocamento ou tremor.
    Celebration,
}

pub struct AnimEngine {
    effects: EffectManager<()>,
    mode: MotionMode,
    last: Instant,
}

impl AnimEngine {
    pub fn new(mode: MotionMode) -> Self {
        Self {
            effects: EffectManager::default(),
            mode,
            last: Instant::now(),
        }
    }

    pub fn set_mode(&mut self, mode: MotionMode) {
        self.mode = mode;
        if mode == MotionMode::Off {
            // Nada de animações penduradas quando o usuário desliga.
            self.effects = EffectManager::default();
        }
    }

    /// Inicia a animação associada a um momento da UI.
    pub fn play(&mut self, fx_kind: ScreenFx, accent: ratatui::style::Color) {
        if !self.mode.animate() {
            return;
        }
        self.last = Instant::now();
        // duration_scale é um percentual (100 = completo). Esquecer o `/100`
        // transformaria um fade de 1,2s em 1200s de redraw contínuo.
        let scaled = |ms: u32| self.mode.duration_scale() * ms / 100;
        match fx_kind {
            ScreenFx::Welcome => {
                // Apenas fades: efeitos que terminam no estado original do
                // buffer. Cuidado descoberto empiricamente (replay de
                // stream real): `fx::dissolve` sozinho sobre conteúdo
                // estático dissolve o texto PARA O NADA. O par
                // dissolve/coalesce fica reservado para quando existir um
                // inspetor de transição próprio (Fases com exercícios).
                let timer = EffectTimer::from_ms(scaled(1_200), Interpolation::SineOut);
                self.effects
                    .add_effect(fx::fade_from(Color::Black, accent, timer));
            }
            ScreenFx::Enter => {
                // surge do primeiro plano escuro e assenta no conteúdo —
                // auto-restaurável em qualquer situação
                let timer = EffectTimer::from_ms(scaled(400), Interpolation::SineOut);
                self.effects
                    .add_effect(fx::fade_from_fg(Color::Black, timer));
            }
            ScreenFx::Leave => {
                let timer = EffectTimer::from_ms(scaled(250), Interpolation::SineIn);
                self.effects.add_effect(fx::fade_from_fg(accent, timer));
            }
            ScreenFx::Celebration => {
                // Acento visual breve e estático; modo Reduced encurta para
                // 40%, e Off não agenda nenhum efeito.
                let timer = EffectTimer::from_ms(scaled(500), Interpolation::SineOut);
                self.effects.add_effect(fx::fade_from_fg(accent, timer));
            }
        }
    }

    /// Deve o loop principal redesenhar neste instante?
    pub fn is_running(&self) -> bool {
        self.effects.is_running()
    }

    /// Avança os efeitos; chamar a cada frame, dentro do draw.
    pub fn tick(&mut self, buf: &mut Buffer, area: Rect) {
        let elapsed = self.last.elapsed().as_millis() as u64;
        self.last = Instant::now();
        // Em `Off`, `play` nunca adiciona efeitos, então `is_running()` é
        // falso e isto praticamente nunca roda trabalho gráfico.
        if self.mode.animate() {
            let ms = u32::try_from(elapsed).unwrap_or(u32::MAX);
            self.effects
                .process_effects(tachyonfx::Duration::from_millis(ms), buf, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modo_off_nao_anima() {
        let mut e = AnimEngine::new(MotionMode::Off);
        e.play(ScreenFx::Welcome, Color::Cyan);
        assert!(!e.is_running());
    }

    #[test]
    fn modo_full_inicia_animacao() {
        let mut e = AnimEngine::new(MotionMode::Full);
        e.play(ScreenFx::Welcome, Color::Cyan);
        assert!(e.is_running());
    }

    #[test]
    fn trocar_para_off_limpa_fila() {
        let mut e = AnimEngine::new(MotionMode::Full);
        e.play(ScreenFx::Welcome, Color::Cyan);
        assert!(e.is_running());
        e.set_mode(MotionMode::Off);
        assert!(!e.is_running());
    }

    #[test]
    fn celebracao_respeita_reduced_e_off() {
        let mut reduced = AnimEngine::new(MotionMode::Reduced);
        reduced.play(ScreenFx::Celebration, Color::Yellow);
        assert!(reduced.is_running());
        let mut off = AnimEngine::new(MotionMode::Off);
        off.play(ScreenFx::Celebration, Color::Yellow);
        assert!(!off.is_running());
    }

    #[test]
    #[ignore = "verificação de tempo real: demora ~2,5s"]
    fn welcome_completa_e_parar_de_rodar() {
        // Regressão: já houve bug em que o multiplicador de escala virava
        // a animação de 1,8s em 1800s. Com Full, 2,5s depois o motor DEVE
        // estar ocioso (senão o app redesenha para sempre).
        let mut e = AnimEngine::new(MotionMode::Full);
        e.play(ScreenFx::Welcome, Color::Cyan);
        let mut buf = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 95, 30));
        let area = buf.area;
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_millis(2_500) {
            std::thread::sleep(std::time::Duration::from_millis(40));
            e.tick(&mut buf, area);
        }
        assert!(!e.is_running(), "efeitos não terminaram no tempo previsto");
    }
}
