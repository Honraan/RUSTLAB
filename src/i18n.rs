//! Strings da interface. Conteúdo voltado a iniciantes começa em
//! português do Brasil (requisito da especificação, seção 2).
//!
//! Centralizar as chaves aqui mantém as telas limpas e torna a
//! tradução futura (fase do requisito 14) uma mudança apenas neste arquivo.

use std::collections::HashMap;
use std::sync::OnceLock;

fn strings() -> &'static HashMap<&'static str, &'static str> {
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

/// Retorna a mensagem para `key`. Se a chave não existir, devolve a própria
/// chave — visível o suficiente para ser pega em testes e no uso.
pub fn t(key: &'static str) -> &'static str {
    strings().get(key).copied().unwrap_or(key)
}

/// Formata `{}` dentro do template identificado por `key`.
pub fn tf(key: &'static str, args: &[&dyn std::fmt::Display]) -> String {
    let template = t(key);
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
        // As chaves listadas em `strings()` devem resolver para si mesmas
        // (isto é, não cair no fallback de "chave ausente").
        let keys = [
            "app.tagline",
            "menu.title",
            "menu.learn",
            "settings.title",
            "help.title",
            "welcome.prompt",
        ];
        for k in keys {
            assert!(t(k) != k, "chave faltando: {k}");
        }
    }

    #[test]
    fn interpolacao_simples() {
        let table: HashMap<&str, &str> = HashMap::from([("x", "a {} b {} c")]);
        // reuso da lógica de tf com uma tabela local via closure não é possível;
        // testamos tf com uma chave real + placeholder embutido.
        let _ = table;
        let s = {
            // Chave temporária não existe; t() devolve a chave, que contém {}.
            tf("template {} de teste {}", &[&"1", &"2"])
        };
        assert_eq!(s, "template 1 de teste 2");
    }
}
