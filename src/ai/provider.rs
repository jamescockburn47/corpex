use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::AiProviderConfig;

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,   // "user" | "assistant" | "system"
    pub content: String,
}

/// Token usage returned from an AI API call.
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl TokenUsage {
    /// Calculate cost in USD based on model pricing.
    /// Claude Haiku 4.5: $0.80/M input, $4.00/M output
    pub fn cost_usd(&self, model: &str) -> f64 {
        let (input_rate, output_rate) = if model.contains("haiku") {
            (0.80, 4.00) // per million tokens
        } else if model.contains("sonnet") {
            (3.00, 15.00)
        } else if model.contains("opus") {
            (15.00, 75.00)
        } else if model.contains("gpt-4o-mini") {
            (0.15, 0.60)
        } else if model.contains("gpt-4o") {
            (2.50, 10.00)
        } else {
            (1.00, 3.00) // default estimate
        };
        (self.input_tokens as f64 * input_rate / 1_000_000.0)
            + (self.output_tokens as f64 * output_rate / 1_000_000.0)
    }
}

/// Result from an AI API call — includes the text and token usage.
#[derive(Debug, Clone)]
pub struct ChatResult {
    pub text: String,
    pub usage: TokenUsage,
    pub model: String,
}

/// Send a conversation to the configured AI provider and get the response.
pub async fn chat_completion(
    config: &AiProviderConfig,
    system_prompt: &str,
    messages: &[ChatMessage],
) -> Result<ChatResult> {
    match config {
        AiProviderConfig::Anthropic { api_key, model } => {
            anthropic_chat(api_key, model, system_prompt, messages).await
        }
        AiProviderConfig::OpenAi { api_key, model } => {
            openai_chat(api_key, model, "https://api.openai.com/v1", system_prompt, messages).await
        }
        AiProviderConfig::Custom { api_key, model, base_url } => {
            openai_chat(api_key, model, base_url, system_prompt, messages).await
        }
        AiProviderConfig::None => {
            anyhow::bail!("No AI provider configured")
        }
    }
}

// ── Anthropic Messages API ───────────────────────────────────────────

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

async fn anthropic_chat(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: &[ChatMessage],
) -> Result<ChatResult> {
    let client = reqwest::Client::new();

    let api_messages: Vec<AnthropicMessage> = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| AnthropicMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    let request = AnthropicRequest {
        model: model.to_string(),
        max_tokens: 4096,
        system: system_prompt.to_string(),
        messages: api_messages,
    };

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Anthropic API request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Anthropic returned {}: {}", status, body);
    }

    let response: AnthropicResponse = resp
        .json()
        .await
        .context("Failed to parse Anthropic response")?;

    let text = response
        .content
        .first()
        .and_then(|c| c.text.clone())
        .unwrap_or_else(|| "No response text".to_string());

    let usage = TokenUsage {
        input_tokens: response.usage.as_ref().and_then(|u| u.input_tokens).unwrap_or(0),
        output_tokens: response.usage.as_ref().and_then(|u| u.output_tokens).unwrap_or(0),
    };

    Ok(ChatResult {
        text,
        usage,
        model: model.to_string(),
    })
}

// ── OpenAI-compatible API ────────────────────────────────────────────

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageResp,
}

#[derive(Deserialize)]
struct OpenAiMessageResp {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

async fn openai_chat(
    api_key: &str,
    model: &str,
    base_url: &str,
    system_prompt: &str,
    messages: &[ChatMessage],
) -> Result<ChatResult> {
    let client = reqwest::Client::new();

    let mut api_messages = vec![OpenAiMessage {
        role: "system".to_string(),
        content: system_prompt.to_string(),
    }];

    for msg in messages {
        if msg.role != "system" {
            api_messages.push(OpenAiMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }
    }

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let request = OpenAiRequest {
        model: model.to_string(),
        messages: api_messages,
        max_tokens: Some(4096),
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .context("OpenAI API request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("OpenAI returned {}: {}", status, body);
    }

    let response: OpenAiResponse = resp
        .json()
        .await
        .context("Failed to parse OpenAI response")?;

    let text = response
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response text".to_string());

    let usage = TokenUsage {
        input_tokens: response.usage.as_ref().and_then(|u| u.prompt_tokens).unwrap_or(0),
        output_tokens: response.usage.as_ref().and_then(|u| u.completion_tokens).unwrap_or(0),
    };

    Ok(ChatResult {
        text,
        usage,
        model: model.to_string(),
    })
}
