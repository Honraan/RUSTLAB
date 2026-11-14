//! Paletas e estilos. Cada tema define *papéis* semânticos (acento, texto,
//! sucesso, erro, bordas...) para que as telas nunca codifiquem cores fixas.
//!
//! O tema `Mono` é a opção acessível exigida pela seção 14: não depende
//! de cor para transmitir informação, apenas de peso/traçado.

use ratatui::style::{Color, Modifier, Style};

use crate::config::ThemeId;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub accent: Color,
    pub accent_dim: Color,
    pub text: Color,
    pub text_dim: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub bold: bool,
    pub underline: bool,
}

impl Theme {
    pub fn for_id(id: ThemeId) -> Self {
        match id {
            ThemeId::Cuia => Self {
                accent: Color::Rgb(222, 165, 132), // cobre da ferrugem
                accent_dim: Color::Rgb(158, 106, 76),
                text: Color::Rgb(235, 229, 221),
                text_dim: Color::Rgb(140, 135, 128),
                success: Color::Rgb(140, 199, 122),
                warning: Color::Rgb(229, 192, 123),
                error: Color::Rgb(224, 108, 117),
                border: Color::Rgb(84, 78, 72),
                border_focused: Color::Rgb(222, 165, 132),
                selection_bg: Color::Rgb(56, 50, 45),
                bold: true,
                underline: false,
            },
            ThemeId::Ferrugem => Self {
                accent: Color::LightRed,
                accent_dim: Color::Red,
                text: Color::White,
                text_dim: Color::DarkGray,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::LightRed,
                border: Color::DarkGray,
                border_focused: Color::LightRed,
                selection_bg: Color::Black,
                bold: true,
                underline: false,
            },
            ThemeId::Mono => Self {
                // Nenhum lugar da UI deve depender dessas cores para sentido;
                // Mono usa Reset para herdar o contraste escolhido pelo usuário.
                accent: Color::Reset,
                accent_dim: Color::Reset,
                text: Color::Reset,
                text_dim: Color::Reset,
                success: Color::Reset,
                warning: Color::Reset,
                error: Color::Reset,
                border: Color::Reset,
                border_focused: Color::Reset,
                selection_bg: Color::Reset,
                bold: true,
                underline: true,
            },
            ThemeId::Oceano => Self {
                accent: Color::Rgb(102, 194, 214),
                accent_dim: Color::Rgb(61, 133, 158),
                text: Color::Rgb(226, 238, 244),
                text_dim: Color::Rgb(132, 154, 168),
                success: Color::Rgb(126, 209, 156),
                warning: Color::Rgb(235, 198, 118),
                error: Color::Rgb(230, 115, 122),
                border: Color::Rgb(68, 92, 108),
                border_focused: Color::Rgb(102, 194, 214),
                selection_bg: Color::Rgb(34, 52, 66),
                bold: true,
                underline: false,
            },
        }
    }

    pub fn base(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn dim(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    pub fn title(&self) -> Style {
        let mut s = Style::default().fg(self.accent);
        if self.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        s
    }

    pub fn selection(&self) -> Style {
        let mut s = Style::default().fg(self.accent).bg(self.selection_bg);
        if self.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        if self.underline {
            s = s.add_modifier(Modifier::UNDERLINED);
        }
        s
    }

    pub fn border_style(&self, focused: bool) -> Style {
        Style::default().fg(if focused {
            self.border_focused
        } else {
            self.border
        })
    }

    pub fn success(&self) -> Style {
        let mut s = Style::default().fg(self.success);
        if self.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        s
    }

    pub fn warning(&self) -> Style {
        Style::default().fg(self.warning)
    }

    pub fn error(&self) -> Style {
        let mut s = Style::default().fg(self.error);
        if self.bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_nao_usa_cores() {
        let t = Theme::for_id(ThemeId::Mono);
        assert_eq!(t.accent, Color::Reset);
        assert_eq!(t.text, Color::Reset);
        assert!(
            t.underline,
            "Mono precisa de sublinhado como pista não-cromática"
        );
    }

    #[test]
    fn todos_os_temas_tem_estilos() {
        for id in ThemeId::ALL {
            let t = Theme::for_id(id);
            // Chama cada construtor de estilo para garantir que nenhum panica.
            let _ = (t.base(), t.dim(), t.title(), t.selection());
            let _ = (
                t.border_style(true),
                t.border_style(false),
                t.success(),
                t.warning(),
            );
        }
    }
}
