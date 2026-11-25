//! Transporte HTTPS do mentor via curl descoberto no PATH.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub(super) fn http_curl_https_for(
    method: &str,
    url: &str,
    bearer_token: Option<&str>,
    json_body: Option<&str>,
    max_time_secs: u32,
    english: bool,
) -> Result<String, String> {
    let mut body_path: Option<PathBuf> = None;
    let mut config = format!(
        "url = \"{}\"\nrequest = \"{}\"\nsilent\nshow-error\nfail-with-body\nmax-time = {}\n",
        url.replace('"', ""),
        method,
        max_time_secs,
    );
    if let Some(body) = json_body {
        let p = temp_payload_file();
        if let Err(err) = std::fs::write(&p, body.as_bytes()) {
            return Err(if english {
                format!("failed to prepare JSON body: {err}")
            } else {
                format!("falha ao preparar corpo JSON: {err}")
            });
        }
        let normalized = p.to_string_lossy().replace('\\', "/");
        config.push_str("header = \"Content-Type: application/json\"\n");
        config.push_str(&format!("data-binary = \"@{normalized}\"\n"));
        body_path = Some(p);
    }
    if let Some(tok) = bearer_token {
        config.push_str(&format!("header = \"Authorization: Bearer {tok}\"\n"));
    }

    let mut child = Command::new("curl")
        .args(["--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            if let Some(p) = &body_path {
                let _ = std::fs::remove_file(p);
            }
            if english {
                format!("could not start curl for HTTPS: {err}")
            } else {
                format!("não consegui iniciar curl para HTTPS: {err}")
            }
        })?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(config.as_bytes());
    }
    let output = child.wait_with_output();
    if let Some(p) = &body_path {
        let _ = std::fs::remove_file(p);
    }

    let output = output.map_err(|err| {
        if english {
            format!("error waiting for curl: {err}")
        } else {
            format!("erro aguardando curl: {err}")
        }
    })?;
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if err_msg.is_empty() {
            if english {
                format!("curl failed with exit code {:?}", output.status.code())
            } else {
                format!("curl falhou com código {:?}", output.status.code())
            }
        } else {
            err_msg
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn temp_payload_file() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "rustlab-mentor-{}-{nanos}.json",
        std::process::id()
    ))
}
