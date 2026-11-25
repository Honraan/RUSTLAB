use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use rustlab::config::{AiProvider, RemoteAiConsent};
use rustlab::mentor::{
    MentorAction, MentorContext, build_agent_messages_for, build_prompt_for,
    extract_rust_code_block, is_learn_mode_context, local_agent_response_for,
    local_pedagogical_response_for, sanitize_learn_mode_response_for,
};

use crate::error::{CmdResult, CommandError};
use crate::state::AppState;

const KEYRING_SERVICE: &str = "org.rustlab.desktop";
const KEYRING_USER: &str = "remote-ai-api-key";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MentorAskRequest {
    pub learn_mode: bool,
    pub action: String,
    pub origin: String,
    pub topic_title: Option<String>,
    pub summary: String,
    pub code: String,
    pub errors: String,
    pub console: String,
    pub question: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MentorReplyDto {
    pub source: String,
    pub body: String,
    pub read_only_enforced: bool,
    pub proposed_code: Option<String>,
    pub probabilistic_notice: String,
}

fn parse_action(s: &str) -> MentorAction {
    match s {
        "explain-concept" => MentorAction::ExplainConcept,
        "explain-error" => MentorAction::ExplainCompilerError,
        "review-code" => MentorAction::ReviewCode,
        "next-hint" => MentorAction::NextStepHint,
        "refactor" => MentorAction::SuggestRefactoring,
        _ => MentorAction::CustomQuestion,
    }
}

pub fn set_api_key(api_key: &str) -> CmdResult<()> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(CommandError::invalid(
            "A chave de API não pode estar vazia.",
        ));
    }
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| {
        CommandError::io("Falha ao acessar Gerenciador de Credenciais do Windows", e)
    })?;
    entry
        .set_password(trimmed)
        .map_err(|e| CommandError::io("Falha ao gravar chave no Windows Credential Manager", e))?;
    Ok(())
}

pub fn delete_api_key() -> CmdResult<()> {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        let _ = entry.delete_credential();
    }
    Ok(())
}

pub fn has_api_key() -> bool {
    if std::env::var("RUSTLAB_AI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .is_ok_and(|v| !v.trim().is_empty())
    {
        return true;
    }
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .ok()
        .and_then(|e| e.get_password().ok())
        .is_some_and(|v| !v.trim().is_empty())
}

fn load_api_key() -> Option<String> {
    if let Ok(v) = std::env::var("RUSTLAB_AI_API_KEY").or_else(|_| std::env::var("OPENAI_API_KEY"))
        && !v.trim().is_empty()
    {
        return Some(v.trim().to_string());
    }
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .ok()
        .and_then(|e| e.get_password().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub async fn list_ollama_models(state: &AppState) -> CmdResult<Vec<String>> {
    let ollama_url = {
        let inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        inner.config.ai.ollama_url.clone()
    };
    let endpoint = format!("{}/api/tags", ollama_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| CommandError::io("Erro ao criar cliente HTTP", e))?;
    let res = client
        .get(&endpoint)
        .send()
        .await
        .map_err(|e| CommandError::io("Ollama não respondeu localmente", e))?;
    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| CommandError::io("Resposta JSON inválida do Ollama", e))?;
    let mut names = Vec::new();
    if let Some(arr) = json.get("models").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(n) = item
                .get("name")
                .or_else(|| item.get("model"))
                .and_then(|v| v.as_str())
                && !names.iter().any(|x| x == n)
            {
                names.push(n.to_string());
            }
        }
    }
    Ok(names)
}

pub async fn mentor_ask(state: &AppState, req: MentorAskRequest) -> CmdResult<MentorReplyDto> {
    let (ai, en, origin_language, recurring_errors) = {
        let inner = state
            .inner
            .lock()
            .map_err(|_| CommandError::new("INTERNAL", "Falha de lock interno", None))?;
        (
            inner.config.ai.clone(),
            inner.is_english(),
            inner.config.origin_language,
            inner.progress.top_recurring_errors(3),
        )
    };

    let safe_origin = if req.learn_mode {
        if req.origin.starts_with("Modo Criar") || req.origin.starts_with("Create Mode") {
            format!("Lição: {}", req.origin)
        } else {
            req.origin.clone()
        }
    } else if req.origin.starts_with("Modo Criar") || req.origin.starts_with("Create Mode") {
        req.origin.clone()
    } else {
        format!("Modo Criar: {}", req.origin)
    };

    let ctx = MentorContext {
        origin: safe_origin,
        topic_title: req.topic_title,
        summary: req.summary,
        code: req.code,
        errors: req.errors,
        console: req.console,
        origin_language,
        recurring_errors,
    };

    let action = parse_action(&req.action);
    let read_only = req.learn_mode || is_learn_mode_context(&ctx);
    let fallback_raw = if read_only {
        local_pedagogical_response_for(action, &ctx, &req.question, en)
    } else {
        local_agent_response_for(&ctx, &req.question, en)
    };

    let (source, raw_body) = match ai.provider {
        AiProvider::Disabled => (
            "Guia local determinístico (offline)".to_string(),
            fallback_raw,
        ),
        AiProvider::Ollama => {
            let messages = if read_only {
                let (sys, usr) = build_prompt_for(action, &ctx, &req.question, en);
                vec![("system".to_string(), sys), ("user".to_string(), usr)]
            } else {
                build_agent_messages_for(&[], &ctx, &req.question, en)
            };
            match query_ollama_http(&ai.ollama_url, &ai.ollama_model, &messages).await {
                Ok(text) => (format!("Ollama ({})", ai.ollama_model), text),
                Err(err) => (
                    format!("Guia local determinístico (fallback — Ollama indisponível: {err})"),
                    fallback_raw,
                ),
            }
        }
        AiProvider::Remote => {
            if ai.remote_consent != RemoteAiConsent::Granted {
                return Err(CommandError::consent(
                    "Provedor remoto selecionado, mas o consentimento explícito para envio externo ainda não foi concedido.",
                ));
            }
            let Some(token) = load_api_key() else {
                return Err(CommandError::consent(
                    "Nenhuma chave de API configurada no Windows Credential Manager.",
                ));
            };
            let messages = if read_only {
                let (sys, usr) = build_prompt_for(action, &ctx, &req.question, en);
                vec![("system".to_string(), sys), ("user".to_string(), usr)]
            } else {
                build_agent_messages_for(&[], &ctx, &req.question, en)
            };
            match query_remote_http(&ai.remote_url, &ai.remote_model, &token, &messages).await {
                Ok(text) => (format!("API Remota ({})", ai.remote_model), text),
                Err(err) => (
                    format!("Guia local determinístico (fallback — erro na API remota: {err})"),
                    fallback_raw,
                ),
            }
        }
    };

    // Garantia técnica obrigatória: no Modo Aprender nenhum bloco de código passa.
    let _ = AtomicBool::new(false).load(Ordering::Relaxed);
    let (body, proposed_code) = if read_only {
        (sanitize_learn_mode_response_for(&raw_body, en), None)
    } else {
        let code_block = extract_rust_code_block(&raw_body);
        (raw_body, code_block)
    };

    Ok(MentorReplyDto {
        source,
        body,
        read_only_enforced: read_only,
        proposed_code,
        probabilistic_notice: "Sugestões do mentor não substituem `cargo check` / `cargo test`: o compilador e os testes determinísticos têm a palavra final.".to_string(),
    })
}

async fn query_ollama_http(
    base_url: &str,
    model: &str,
    messages: &[(String, String)],
) -> Result<String, String> {
    let endpoint = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": model,
        "stream": false,
        "messages": messages.iter().map(|(r, c)| serde_json::json!({"role": r, "content": c})).collect::<Vec<_>>()
    });
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .post(&endpoint)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("HTTP {}", res.status()));
    }
    let val: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    val.get("message")
        .and_then(|m| m.get("content"))
        .or_else(|| val.get("response"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "Resposta vazia do Ollama".to_string())
}

async fn query_remote_http(
    base_url: &str,
    model: &str,
    token: &str,
    messages: &[(String, String)],
) -> Result<String, String> {
    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "messages": messages.iter().map(|(r, c)| serde_json::json!({"role": r, "content": c})).collect::<Vec<_>>()
    });
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .post(&endpoint)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("HTTP {}", res.status()));
    }
    let val: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    val.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "Resposta remota sem choices[0].message.content".to_string())
}
