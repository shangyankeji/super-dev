//! Direct-HTTP runtimes — call an LLM API endpoint without a host CLI.
//!
//! These are the "custom model" path. A user who does not have (or does not
//! want) a logged-in `claude` / `codex` CLI can instead point Super Dev at an
//! HTTP endpoint with an API key, exactly like Cline's "OpenAI Compatible"
//! provider or Aider's `--model` + `OPENAI_API_BASE`.
//!
//! Two runtimes, two wire protocols:
//!
//! - [`OpenAiHttpRuntime`] — `POST {base_url}/chat/completions`. The de-facto
//!   standard spoken by OpenAI, DeepSeek, OpenRouter, Together, Groq, Mistral,
//!   vLLM, LM Studio, Ollama (`/v1`), 智谱, 阿里百炼, 火山引擎, … One runtime
//!   covers the entire "BYO key" ecosystem.
//! - [`AnthropicHttpRuntime`] — `POST {base_url}/v1/messages` (native Anthropic
//!   Messages API). Kept as a distinct runtime because Anthropic's API has a
//!   first-class `system` field and a different auth header (`x-api-key` +
//!   `anthropic-version`), so modelling it as "OpenAI compatible" loses fidelity.
//!
//! Both implement [`Runtime`] so the existing `AgentRunner` drives them
//! unchanged — a custom API endpoint is just another "prompt in, text out"
//! backend, same as a host CLI.
//!
//! ## API-key resolution
//!
//! [`expand_env`] turns `"${DEEPSEEK_API_KEY}"` into the value of that env var,
//! and leaves a bare key (`"sk-..."`) untouched. This lets users keep real keys
//! out of their config files while still writing a literal for quick experiments.
//!
//! [`Runtime`]: crate::Runtime

use std::sync::OnceLock;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;

use crate::{CompletionRequest, CompletionResponse, Runtime, RuntimeError, RuntimeKind, Usage};

/// Default per-call timeout for a direct-API completion. Mirrors the host-CLI
/// default so a slow model and a slow CLI are treated the same way.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// A pooled [`reqwest::Client`] reused across every HTTP-completion call so we
/// don't pay a fresh TLS handshake per phase. Built lazily on first use.
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn shared_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

/// Expand a `"${ENV_VAR}"` reference into that env var's value. A bare value
/// (no surrounding `${…}`) is returned unchanged. An unset/empty env var yields
/// an empty string — callers surface that as a config error rather than
/// silently sending an empty bearer token.
///
/// # Examples
/// ```
/// # use super_dev_runtime::http::expand_env;
/// // Bare key passes through untouched.
/// assert_eq!(expand_env("sk-deadbeef"), "sk-deadbeef");
/// ```
#[must_use]
pub fn expand_env(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("${") && trimmed.ends_with('}') && trimmed.len() > 3 {
        let name = &trimmed[2..trimmed.len() - 1];
        return std::env::var(name).unwrap_or_default();
    }
    raw.to_string()
}

/// Shared metadata every direct-API runtime carries. Kept as a plain struct
/// (not an enum) so the two runtimes differ only in `complete()`.
#[derive(Debug, Clone)]
struct HttpEndpoint {
    /// Base URL up to and excluding the path, e.g.
    /// `https://api.deepseek.com` or `https://api.anthropic.com`.
    /// A trailing slash is tolerated and normalised away.
    base_url: String,
    /// Raw credential — may be a bare key or a `${VAR}` reference. Resolved
    /// per-call via [`expand_env`] so a rotated env var takes effect without a
    /// restart.
    api_key: String,
    /// Model identifier sent in the request body.
    model: String,
    /// Audit label for [`CompletionResponse::id`] — e.g. `"openai-http"` /
    /// `"anthropic-http"`. Lets the audit chain tell direct-API runs apart
    /// from host-CLI runs.
    id_label: &'static str,
}

impl HttpEndpoint {
    /// Join `base` with a path, collapsing any accidental double slash.
    fn url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        // `path` is expected to start with '/' (e.g. "/chat/completions").
        format!("{base}{path}")
    }

    /// Resolve the api key via [`expand_env`] and error if it came back empty.
    fn resolved_key(&self) -> Result<String, RuntimeError> {
        let key = expand_env(&self.api_key);
        if key.trim().is_empty() {
            return Err(RuntimeError::Config(format!(
                "api_key for model `{}` resolved to empty — set the referenced env var or write a literal key",
                self.model
            )));
        }
        Ok(key)
    }
}

// ---------------------------------------------------------------------------
// OpenAI-compatible runtime
// ---------------------------------------------------------------------------

/// A runtime that talks the OpenAI Chat Completions wire protocol.
///
/// Construct with [`OpenAiHttpRuntime::new`], passing the base URL (no
/// `/chat/completions` suffix), the API key, and the model id. One runtime
/// covers every OpenAI-compatible provider.
///
/// ```toml
/// # ~/.super-dev/config.toml — what the TUI writes from `/provider add`
/// [providers.deepseek]
/// kind = "openai"
/// base_url = "https://api.deepseek.com/v1"
/// api_key = "${DEEPSEEK_API_KEY}"
/// model = "deepseek-chat"
/// ```
pub struct OpenAiHttpRuntime {
    ep: HttpEndpoint,
}

impl OpenAiHttpRuntime {
    /// Build an OpenAI-compatible runtime. `base_url` should be the API root
    /// (e.g. `https://api.openai.com/v1` or `https://api.deepseek.com/v1`);
    /// `/chat/completions` is appended automatically.
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            ep: HttpEndpoint {
                base_url: base_url.into(),
                api_key: api_key.into(),
                model: model.into(),
                id_label: "openai-http",
            },
        }
    }
}

#[async_trait]
impl Runtime for OpenAiHttpRuntime {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::Openai
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, RuntimeError> {
        let key = self.ep.resolved_key()?;
        // OpenAI's schema has no first-class system role in the body — we merge
        // the system prompt into the messages array as a leading system message
        // (most compatible servers honour it).
        let mut messages: Vec<serde_json::Value> = Vec::with_capacity(req.messages.len() + 1);
        if let Some(sys) = &req.system {
            if !sys.trim().is_empty() {
                messages.push(serde_json::json!({"role": "system", "content": sys}));
            }
        }
        for m in &req.messages {
            messages.push(serde_json::json!({"role": m.role, "content": m.content}));
        }
        let mut body = serde_json::json!({
            "model": self.ep.model,
            "messages": messages,
        });
        if let Some(mt) = req.max_tokens {
            body["max_tokens"] = serde_json::json!(mt);
        }
        if let Some(t) = req.temperature {
            body["temperature"] = serde_json::json!(t);
        }

        let url = self.ep.url("/chat/completions");
        let resp = shared_client()
            .post(&url)
            .bearer_auth(&key)
            .json(&body)
            .send()
            .await
            .map_err(|e| map_reqwest_error(&url, &e))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(RuntimeError::HostProcess(format!(
                "OpenAI-compatible endpoint {url} returned {status}: {}",
                truncate(&text, 1024)
            )));
        }
        let parsed: OpenAiResponse = resp.json().await.map_err(|e| {
            RuntimeError::Parse(serde_json::Error::io(std::io::Error::other(format!(
                "decode {url} body: {e}"
            ))))
        })?;
        let text = parsed
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        let usage = parsed
            .usage
            .map(|u| Usage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
            })
            .unwrap_or_default();
        tracing::debug!(
            url = %url,
            model = %self.ep.model,
            in_tokens = usage.input_tokens,
            out_tokens = usage.output_tokens,
            "openai-compatible completion ok"
        );
        Ok(CompletionResponse {
            text,
            id: self.ep.id_label.to_string(),
            model: self.ep.model.clone(),
            usage,
        })
    }
}

/// Subset of the OpenAI chat-completion response we decode. Unknown fields are
/// ignored so provider-specific extensions (DeepSeek's `prompt_cache_hit_tokens`
/// etc.) never break parsing.
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    #[serde(default)]
    message: OpenAiMessage,
}

#[derive(Debug, Default, Deserialize)]
struct OpenAiMessage {
    #[serde(default)]
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

// ---------------------------------------------------------------------------
// Anthropic native runtime
// ---------------------------------------------------------------------------

/// A runtime that talks Anthropic's native Messages API (`POST /v1/messages`).
///
/// Kept separate from [`OpenAiHttpRuntime`] because the protocol differs in
/// load-bearing ways: a first-class `system` field, `x-api-key` auth, the
/// mandatory `anthropic-version` header, and `content` returned as an array of
/// typed blocks. Collapsing it into "OpenAI compatible" would drop the system
/// prompt and mis-parse the response.
///
/// ```toml
/// [providers.anthropic]
/// kind = "anthropic"
/// base_url = "https://api.anthropic.com"
/// api_key = "${ANTHROPIC_API_KEY}"
/// model = "claude-sonnet-4-20250514"
/// ```
pub struct AnthropicHttpRuntime {
    ep: HttpEndpoint,
}

impl AnthropicHttpRuntime {
    /// Build an Anthropic-native runtime. `base_url` is the API root (e.g.
    /// `https://api.anthropic.com`); `/v1/messages` is appended automatically.
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            ep: HttpEndpoint {
                base_url: base_url.into(),
                api_key: api_key.into(),
                model: model.into(),
                id_label: "anthropic-http",
            },
        }
    }
}

#[async_trait]
impl Runtime for AnthropicHttpRuntime {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::Anthropic
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, RuntimeError> {
        let key = self.ep.resolved_key()?;
        // Anthropic requires max_tokens; default to a generous cap so a caller
        // that didn't set one (host-CLI callers never do) still gets a full
        // answer. The agent's phase helpers pass explicit caps where it matters.
        let max_tokens = req.max_tokens.unwrap_or(4096);
        let mut body = serde_json::json!({
            "model": self.ep.model,
            "max_tokens": max_tokens,
            "messages": req.messages.iter().map(|m| {
                serde_json::json!({"role": m.role, "content": m.content})
            }).collect::<Vec<_>>(),
        });
        if let Some(sys) = &req.system {
            if !sys.trim().is_empty() {
                body["system"] = serde_json::json!(sys);
            }
        }
        if let Some(t) = req.temperature {
            body["temperature"] = serde_json::json!(t);
        }

        let url = self.ep.url("/v1/messages");
        let resp = shared_client()
            .post(&url)
            .header("x-api-key", &key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| map_reqwest_error(&url, &e))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(RuntimeError::HostProcess(format!(
                "Anthropic endpoint {url} returned {status}: {}",
                truncate(&text, 1024)
            )));
        }
        let parsed: AnthropicResponse = resp.json().await.map_err(|e| {
            RuntimeError::Parse(serde_json::Error::io(std::io::Error::other(format!(
                "decode {url} body: {e}"
            ))))
        })?;
        let text = parsed
            .content
            .iter()
            .filter_map(|b| {
                if b.kind.as_deref() == Some("text") {
                    Some(b.text.clone().unwrap_or_default())
                } else {
                    None
                }
            })
            .collect::<String>();
        let usage = Usage {
            input_tokens: parsed.usage.input_tokens,
            output_tokens: parsed.usage.output_tokens,
        };
        tracing::debug!(
            url = %url,
            model = %self.ep.model,
            in_tokens = usage.input_tokens,
            out_tokens = usage.output_tokens,
            "anthropic-native completion ok"
        );
        Ok(CompletionResponse {
            text,
            id: self.ep.id_label.to_string(),
            model: self.ep.model.clone(),
            usage,
        })
    }
}

/// Subset of Anthropic's Messages response.
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<AnthropicBlock>,
    #[serde(default)]
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Turn a reqwest transport error into a [`RuntimeError`], detecting the
/// timeout case so callers can retry on timeout (matching the host-CLI path).
fn map_reqwest_error(url: &str, e: &reqwest::Error) -> RuntimeError {
    if e.is_timeout() {
        RuntimeError::Timeout(DEFAULT_TIMEOUT.as_secs(), format!("{url}: {e}"))
    } else {
        RuntimeError::HostProcess(format!("{url}: {e}"))
    }
}

/// Truncate `s` to `max` bytes on a UTF-8 char boundary, appending an
/// ellipsis marker when it was cut. Keeps error messages bounded.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut idx = max;
    while !s.is_char_boundary(idx) {
        idx -= 1;
    }
    let mut out = s[..idx].to_string();
    out.push_str("…[truncated]");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompletionRequest, Message};

    fn req(text: &str) -> CompletionRequest {
        CompletionRequest {
            model: "test".into(),
            messages: vec![Message {
                role: "user".into(),
                content: text.into(),
            }],
            max_tokens: Some(64),
            temperature: None,
            system: None,
        }
    }

    #[test]
    fn expand_env_bare_passthrough() {
        assert_eq!(expand_env("sk-deadbeef"), "sk-deadbeef");
        assert_eq!(expand_env("  spaced  "), "  spaced  ");
    }

    #[test]
    fn expand_env_ref_resolves() {
        std::env::set_var("SUPER_DEV_TEST_KEY_XYZ", "resolved-123");
        assert_eq!(expand_env("${SUPER_DEV_TEST_KEY_XYZ}"), "resolved-123");
        std::env::remove_var("SUPER_DEV_TEST_KEY_XYZ");
    }

    #[test]
    fn expand_env_unset_ref_is_empty() {
        // An unset var yields "" — the runtime then turns this into a Config error.
        assert_eq!(expand_env("${SUPER_DEV_DEFINITELY_UNSET_VAR}"), "");
    }

    #[test]
    fn expand_env_malformed_not_a_ref() {
        // Missing closing brace → not a ref, returned as-is.
        assert_eq!(expand_env("${BROKEN"), "${BROKEN");
    }

    #[test]
    fn truncate_keeps_short_strings() {
        assert_eq!(truncate("short", 100), "short");
    }

    #[test]
    fn truncate_cuts_long_strings_on_boundary() {
        let long = "é".repeat(200); // each é is 2 bytes
        let out = truncate(&long, 10);
        assert!(out.ends_with("…[truncated]"));
        // Must be valid UTF-8 (cut landed on a boundary).
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[tokio::test]
    async fn openai_runtime_parses_choice_and_usage() {
        let mut server = mockito::Server::new_async().await;
        let body = serde_json::json!({
            "id": "chatcmpl-x",
            "choices": [{"message": {"role": "assistant", "content": "hello world"}}],
            "usage": {"prompt_tokens": 5, "completion_tokens": 2}
        });
        let _m = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&body).unwrap())
            .create_async()
            .await;
        let rt = OpenAiHttpRuntime::new(server.url(), "sk-test", "deepseek-chat");
        let resp = rt.complete(req("hi")).await.expect("completion ok");
        assert_eq!(resp.text, "hello world");
        assert_eq!(resp.model, "deepseek-chat");
        assert_eq!(resp.id, "openai-http");
        assert_eq!(resp.usage.input_tokens, 5);
        assert_eq!(resp.usage.output_tokens, 2);
    }

    #[tokio::test]
    async fn openai_runtime_merges_system_prompt() {
        let mut server = mockito::Server::new_async().await;
        let body = serde_json::json!({"choices":[{"message":{"content":"ok"}}]});
        let m = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_body(serde_json::to_string(&body).unwrap())
            .create_async()
            .await;
        let rt = OpenAiHttpRuntime::new(server.url(), "sk-test", "m");
        let mut r = req("hi");
        r.system = Some("BE THE SPEC".into());
        let _ = rt.complete(r).await.unwrap();
        // The mock asserts the body contained the system message.
        m.assert_async().await;
    }

    #[tokio::test]
    async fn openai_runtime_surfaces_http_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/chat/completions")
            .with_status(401)
            .with_body("{\"error\":\"bad key\"}")
            .create_async()
            .await;
        let rt = OpenAiHttpRuntime::new(server.url(), "sk-bad", "m");
        let err = rt.complete(req("hi")).await.unwrap_err();
        match err {
            RuntimeError::HostProcess(msg) => {
                assert!(msg.contains("401"), "msg was: {msg}");
            }
            other => panic!("expected HostProcess, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn openai_runtime_errors_on_empty_key() {
        // ${UNSET} resolves to "" → Config error before any HTTP call.
        let rt = OpenAiHttpRuntime::new("https://example.invalid", "${SUPER_DEV_UNSET_K}", "m");
        let err = rt.complete(req("hi")).await.unwrap_err();
        assert!(matches!(err, RuntimeError::Config(_)));
    }

    #[tokio::test]
    async fn anthropic_runtime_parses_text_blocks_and_usage() {
        let mut server = mockito::Server::new_async().await;
        let body = serde_json::json!({
            "id": "msg_x",
            "content": [
                {"type": "text", "text": "part 1 "},
                {"type": "text", "text": "part 2"}
            ],
            "usage": {"input_tokens": 10, "output_tokens": 4}
        });
        let _m = server
            .mock("POST", "/v1/messages")
            .match_header("x-api-key", "sk-ant")
            .match_header("anthropic-version", "2023-06-01")
            .with_status(200)
            .with_body(serde_json::to_string(&body).unwrap())
            .create_async()
            .await;
        let rt = AnthropicHttpRuntime::new(server.url(), "sk-ant", "claude-sonnet-4");
        let resp = rt.complete(req("hi")).await.expect("completion ok");
        assert_eq!(resp.text, "part 1 part 2");
        assert_eq!(resp.id, "anthropic-http");
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(resp.usage.output_tokens, 4);
    }

    #[tokio::test]
    async fn anthropic_runtime_sends_system_field() {
        let mut server = mockito::Server::new_async().await;
        let body = serde_json::json!({"content":[{"type":"text","text":"ok"}],"usage":{}});
        let m = server
            .mock("POST", "/v1/messages")
            .with_status(200)
            .with_body(serde_json::to_string(&body).unwrap())
            .create_async()
            .await;
        let rt = AnthropicHttpRuntime::new(server.url(), "sk-ant", "claude");
        let mut r = req("hi");
        r.system = Some("SYSTEM HERE".into());
        let _ = rt.complete(r).await.unwrap();
        m.assert_async().await;
    }

    #[tokio::test]
    async fn anthropic_runtime_surfaces_http_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/v1/messages")
            .with_status(529)
            .with_body("{\"type\":\"overloaded\"}")
            .create_async()
            .await;
        let rt = AnthropicHttpRuntime::new(server.url(), "sk-ant", "claude");
        let err = rt.complete(req("hi")).await.unwrap_err();
        match err {
            RuntimeError::HostProcess(msg) => assert!(msg.contains("529")),
            other => panic!("expected HostProcess, got {other:?}"),
        }
    }

    #[test]
    fn url_normalises_trailing_slash() {
        let ep = HttpEndpoint {
            base_url: "https://api.x.com/".into(),
            api_key: "k".into(),
            model: "m".into(),
            id_label: "test",
        };
        assert_eq!(
            ep.url("/chat/completions"),
            "https://api.x.com/chat/completions"
        );
    }
}
