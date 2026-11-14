//! Strings da interface com suporte a inglês (`en-US`, padrão inicial)
//! e português do Brasil (`pt-BR`), selecionável nas Configurações.

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

thread_local! {
    static CURRENT_LANG_EN: Cell<bool> = const { Cell::new(false) };
}

pub fn set_english(english: bool) {
    CURRENT_LANG_EN.with(|c| c.set(english));
}

pub fn set_language(lang: &str) {
    let english = !lang.trim().to_ascii_lowercase().starts_with("pt");
    set_english(english);
}

pub fn is_english() -> bool {
    CURRENT_LANG_EN.with(Cell::get)
}

pub fn bi<'a>(pt: &'a str, en: &'a str, english: bool) -> &'a str {
    if english { en } else { pt }
}

fn strings_pt() -> &'static HashMap<&'static str, &'static str> {
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| {
        HashMap::from([
            // Rodapé / geral
            ("app.tagline", "Aprenda Rust fazendo — direto no terminal"),
            ("footer.quit", "q sair"),
            ("footer.enter", "enter confirmar"),
            ("footer.esc", "esc voltar"),
            ("footer.nav", "↑↓ navegar"),
            // Menu principal
            ("menu.title", "RustLab"),
            ("menu.learn", "Modo Aprender"),
            ("menu.create", "Modo Criar"),
            ("menu.progress", "Meu Progresso"),
            ("menu.settings", "Configurações"),
            ("menu.help", "Ajuda & Atalhos"),
            ("menu.quit", "Sair"),
            ("menu.subtitle", "Escolha um caminho para começar"),
            // Telas placeholder
            ("placeholder.heading", "Em construção"),
            ("placeholder.phase", "Em construção — Fase {} do plano"),
            (
                "placeholder.footer",
                "Esta área será implementada nas próximas fases do projeto.",
            ),
            // Configurações
            ("settings.title", "Configurações"),
            (
                "settings.legend",
                "Use {} para navegar e ajustar — tudo salva automaticamente",
            ),
            ("settings.saved", "Configurações salvas"),
            (
                "settings.save_error",
                "Não foi possível salvar as configurações",
            ),
            ("settings.theme", "Tema"),
            ("settings.animations", "Animações"),
            ("settings.sound", "Alertas sonoros"),
            ("settings.volume", "Volume do som de XP"),
            ("settings.gamification", "Efeitos de gamificação"),
            ("settings.language", "Idioma"),
            ("settings.audio_report", "Dispositivo de áudio detectado"),
            ("settings.anim_full", "Completas"),
            ("settings.anim_reduced", "Reduzidas"),
            ("settings.anim_off", "Desligadas"),
            ("settings.gam_on", "Visíveis"),
            ("settings.gam_off", "Ocultas"),
            ("settings.sound_on", "Ativados"),
            ("settings.sound_off", "Desativados"),
            ("settings.ai_provider", "Mentor de IA"),
            ("settings.ollama_model", "Modelo Ollama (local)"),
            ("settings.ai.disabled", "Desativado"),
            // Ajuda
            ("help.title", "Ajuda & Atalhos"),
            (
                "help.intro",
                "RustLab é guiado por teclado. Estas são as teclas disponíveis:",
            ),
            ("help.k_nav", "Navegar"),
            ("help.k_enter", "Enter"),
            ("help.k_adjust", "Ajustar"),
            (
                "help.audio_note",
                "XP: tom curto e suave via paplay (PulseAudio/PipeWire); zero silencia.",
            ),
            // Boas-vindas
            ("welcome.prompt", "Pressione qualquer tecla para começar"),
            ("welcome.caption", "um laboratório de Rust no seu terminal"),
            // Navegador de curso (Fase 2)
            ("browser.title", "Etapas do curso"),
            ("browser.lessons", "lições"),
            ("browser.planned-tag", "planejada"),
            (
                "browser.planned-note",
                "Este capítulo ainda não tem lições escritas.",
            ),
            (
                "browser.planned-line1",
                "Capítulo anunciado no roadmap — ainda sem lições.",
            ),
            (
                "browser.planned-line2",
                "Conteúdo previsto para a Fase {} do projeto.",
            ),
            (
                "learn.empty",
                "Currículo indisponível: conteúdo embutido inválido ou não carregado.",
            ),
            // Lição (Fase 2)
            (
                "lesson.nav-hint",
                "↑↓ rolar · 1-9 responder · d demonstração · ←→ lições · esc voltar",
            ),
            ("lesson.answer-hint", "responda com as teclas 1 a {}"),
            (
                "lesson.try-again",
                "quase! tente outra opção — ler de novo ajuda :)",
            ),
            ("lesson.why-head", "porquê:"),
            ("lesson.progress", "{} · leitura {}%"),
            ("lesson.progress-done", "{} · lição concluída — → avança"),
            (
                "lesson.blocked_next",
                "ainda falta responder o quiz desta lição",
            ),
            (
                "lesson.scroll_to_end",
                "role até o fim da lição (End) para concluí-la",
            ),
            (
                "lesson.end-of-curriculum",
                "Fim do conteúdo disponível até aqui — obrigado por estudar!",
            ),
            (
                "demo.simplified",
                "(ilustração conceitual — simplificada; não é um inspetor de memória)",
            ),
        ])
    })
}

fn strings_en() -> &'static HashMap<&'static str, &'static str> {
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| {
        HashMap::from([
            // Footer / general
            (
                "app.tagline",
                "Learn Rust by doing — right in your terminal",
            ),
            ("footer.quit", "q quit"),
            ("footer.enter", "enter confirm"),
            ("footer.esc", "esc back"),
            ("footer.nav", "↑↓ navigate"),
            // Main menu
            ("menu.title", "RustLab"),
            ("menu.learn", "Learn Mode"),
            ("menu.create", "Create Mode"),
            ("menu.progress", "My Progress"),
            ("menu.settings", "Settings"),
            ("menu.help", "Help & Shortcuts"),
            ("menu.quit", "Quit"),
            ("menu.subtitle", "Choose a path to begin"),
            // Placeholder screens
            ("placeholder.heading", "Under construction"),
            ("placeholder.phase", "Under construction — Phase {} of plan"),
            (
                "placeholder.footer",
                "This area will be implemented in future phases of the project.",
            ),
            // Settings
            ("settings.title", "Settings"),
            (
                "settings.legend",
                "Use {} to navigate and adjust — everything saves automatically",
            ),
            ("settings.saved", "Settings saved"),
            ("settings.save_error", "Could not save settings"),
            ("settings.theme", "Theme"),
            ("settings.animations", "Animations"),
            ("settings.sound", "Sound alerts"),
            ("settings.volume", "XP sound volume"),
            ("settings.gamification", "Gamification effects"),
            ("settings.language", "Language"),
            ("settings.audio_report", "Detected audio device"),
            ("settings.anim_full", "Full"),
            ("settings.anim_reduced", "Reduced"),
            ("settings.anim_off", "Off"),
            ("settings.gam_on", "Visible"),
            ("settings.gam_off", "Hidden"),
            ("settings.sound_on", "Enabled"),
            ("settings.sound_off", "Disabled"),
            ("settings.ai_provider", "AI Mentor"),
            ("settings.ollama_model", "Ollama model (local)"),
            ("settings.ai.disabled", "Disabled"),
            // Help
            ("help.title", "Help & Shortcuts"),
            (
                "help.intro",
                "RustLab is keyboard-driven. These shortcuts are available:",
            ),
            ("help.k_nav", "Navigate"),
            ("help.k_enter", "Enter"),
            ("help.k_adjust", "Adjust"),
            (
                "help.audio_note",
                "XP: short soft tone via paplay (PulseAudio/PipeWire); zero mutes.",
            ),
            // Welcome
            ("welcome.prompt", "Press any key to start"),
            ("welcome.caption", "a Rust lab right in your terminal"),
            // Browser
            ("browser.title", "Course stages"),
            ("browser.lessons", "lessons"),
            ("browser.planned-tag", "planned"),
            (
                "browser.planned-note",
                "This chapter does not have written lessons yet.",
            ),
            (
                "browser.planned-line1",
                "Roadmap chapter — lessons not written yet.",
            ),
            (
                "browser.planned-line2",
                "Content scheduled for Phase {} of the project.",
            ),
            (
                "learn.empty",
                "Curriculum unavailable: embedded content invalid or not loaded.",
            ),
            // Lesson
            (
                "lesson.nav-hint",
                "↑↓ scroll · 1-9 answer · d demo · ←→ lessons · esc back",
            ),
            ("lesson.answer-hint", "answer with keys 1 to {}"),
            (
                "lesson.try-again",
                "almost! try another option — re-reading helps :)",
            ),
            ("lesson.why-head", "why:"),
            ("lesson.progress", "{} · read {}%"),
            ("lesson.progress-done", "{} · lesson complete — → next"),
            (
                "lesson.blocked_next",
                "please answer this lesson's quiz first",
            ),
            (
                "lesson.scroll_to_end",
                "scroll to the end of the lesson (End) to finish it",
            ),
            (
                "lesson.end-of-curriculum",
                "End of available content so far — thanks for studying!",
            ),
            (
                "demo.simplified",
                "(conceptual illustration — simplified; not a memory inspector)",
            ),
        ])
    })
}

pub fn t_for(key: &'static str, english: bool) -> &'static str {
    if english && let Some(v) = strings_en().get(key).copied() {
        return v;
    }
    strings_pt().get(key).copied().unwrap_or(key)
}

/// Retorna a mensagem para `key` no idioma ativo.
pub fn t(key: &'static str) -> &'static str {
    t_for(key, is_english())
}

pub fn tf_for(key: &'static str, english: bool, args: &[&dyn std::fmt::Display]) -> String {
    let template = t_for(key, english);
    format_template(template, args)
}

/// Formata `{}` dentro do template identificado por `key`.
pub fn tf(key: &'static str, args: &[&dyn std::fmt::Display]) -> String {
    format_template(t(key), args)
}

fn format_template(template: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut args = args.iter();
    let mut chars = template.char_indices().peekable();
    while let Some((_, c)) = chars.next() {
        if c == '{' {
            // Consome até '}'
            while let Some(&(_, cc)) = chars.peek() {
                if chars.next().is_some() && cc == '}' {
                    break;
                }
            }
            if let Some(arg) = args.next() {
                out.push_str(&arg.to_string());
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todas_as_chaves_usadas_existem() {
        let keys = [
            "app.tagline",
            "menu.title",
            "menu.learn",
            "settings.title",
            "settings.language",
            "help.title",
            "welcome.prompt",
        ];
        for k in keys {
            assert!(t_for(k, false) != k, "chave pt faltando: {k}");
            assert!(t_for(k, true) != k, "chave en faltando: {k}");
        }
        for k in strings_pt().keys() {
            assert!(
                strings_en().contains_key(k),
                "tradução en-US ausente para chave {k}"
            );
        }
        assert_eq!(t_for("settings.title", true), "Settings");
        assert_eq!(t_for("settings.title", false), "Configurações");
    }

    #[test]
    fn interpolacao_simples() {
        let s = tf("template {} de teste {}", &[&"1", &"2"]);
        assert_eq!(s, "template 1 de teste 2");
        let en = tf_for("lesson.progress", true, &[&"1/3", &"50"]);
        assert_eq!(en, "1/3 · read 50%");
    }
}
