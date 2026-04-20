use crate::constants::{AUDIT_TIMEOUT_MS, AUDIT_URL, TELEMETRY_URL, VERSION};
use crate::types::AuditResponse;
use std::collections::HashMap;
use std::time::Duration;

fn telemetry_url() -> String {
    std::env::var("SKILLS_API_URL")
        .map(|base| format!("{base}/api/telemetry"))
        .unwrap_or_else(|_| TELEMETRY_URL.to_string())
}

fn audit_url() -> String {
    std::env::var("SKILLS_API_URL")
        .map(|base| format!("{base}/api/audit"))
        .unwrap_or_else(|_| AUDIT_URL.to_string())
}

pub fn is_telemetry_disabled() -> bool {
    std::env::var("DISABLE_TELEMETRY").is_ok() || std::env::var("DO_NOT_TRACK").is_ok()
}

fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
}

/// Fire-and-forget telemetry. Never blocks, never fails visibly.
pub fn track(event: &str, params: HashMap<String, String>) {
    if is_telemetry_disabled() {
        return;
    }
    let mut params = params;
    params.insert("event".into(), event.into());
    params.insert("version".into(), VERSION.into());
    if is_ci() {
        params.insert("ci".into(), "1".into());
    }
    tokio::spawn(async move {
        let query = build_query(&params);
        let url = format!("{}?{query}", telemetry_url());
        let _ = crate::http::client().get(&url).send().await;
    });
}

/// Fetch audit data with a timeout. Returns None on any error.
pub async fn fetch_audit_data(source: &str, skill_slugs: &[String]) -> Option<AuditResponse> {
    if is_telemetry_disabled() || skill_slugs.is_empty() {
        return None;
    }

    let url = format!(
        "{}?source={}&skills={}",
        audit_url(),
        urlencoded(source),
        skill_slugs
            .iter()
            .map(|s| urlencoded(s))
            .collect::<Vec<_>>()
            .join(",")
    );

    let result = tokio::time::timeout(
        Duration::from_millis(AUDIT_TIMEOUT_MS),
        crate::http::client().get(&url).send(),
    )
    .await;

    match result {
        Ok(Ok(resp)) if resp.status().is_success() => resp.json().await.ok(),
        _ => None,
    }
}

fn build_query(params: &HashMap<String, String>) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoded(k), urlencoded(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
