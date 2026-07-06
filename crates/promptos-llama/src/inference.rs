use crate::bridge::{InferenceConfig, InferenceOutput};
use regex::Regex;
use std::time::Instant;

const DEFAULT_SYSTEM_PROMPT: &str = "You are a precise, concise assistant.";

pub fn build_chatml_prompt(system: Option<&str>, user: &str) -> String {
    let sys = system.unwrap_or(DEFAULT_SYSTEM_PROMPT);
    format!(
        "<|im_start|>system\n{}\n<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
        sys, user
    )
}

pub fn extract_completion_output(text: &str) -> String {
    let text = text.trim();
    if let Some(pos) = text.rfind("> EOF by user") {
        text[..pos].trim().to_string()
    } else {
        text.to_string()
    }
}

pub fn truncate_to_tokens(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        text.to_string()
    } else {
        let mut end = max_chars;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &text[..end])
    }
}

pub fn run_inference(
    model_path: &str,
    prompt: &str,
    config: &InferenceConfig,
) -> Result<InferenceOutput, String> {
    let start = Instant::now();
    let input_tokens = prompt.split_whitespace().count() as u32;

    if !std::path::Path::new(model_path).exists() {
        return Err(format!("Model not found: {}", model_path));
    }

    let tmpfile = std::env::temp_dir().join(format!("promptos_prompt_{}.txt", std::process::id()));
    std::fs::write(&tmpfile, prompt).map_err(|e| format!("Failed to write prompt file: {}", e))?;

    let mut cmd = std::process::Command::new("llama-completion");
    cmd.arg("-m").arg(model_path);
    cmd.arg("-f").arg(&tmpfile);
    cmd.arg("-n").arg(config.max_tokens.to_string());
    cmd.arg("--temp").arg(config.temperature.to_string());
    cmd.arg("--top-p").arg(config.top_p.to_string());
    cmd.arg("--repeat-penalty").arg(config.repeat_penalty.to_string());
    cmd.arg("--no-display-prompt");
    cmd.arg("-ngl").arg("99");

    if let Some(seed) = config.seed {
        cmd.arg("--seed").arg(seed.to_string());
    }

    let output = cmd.output().map_err(|e| format!("Failed to run llama-completion: {}. Install with: brew install llama.cpp", e))?;

    std::fs::remove_file(&tmpfile).ok();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("llama-completion failed: {}", stderr.trim()));
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let text = extract_completion_output(&raw);
    let elapsed = start.elapsed().as_millis() as u64;
    let out_tokens = text.split_whitespace().count() as u32;

    Ok(InferenceOutput {
        text,
        tokens_generated: out_tokens,
        tokens_input: input_tokens,
        inference_time_ms: elapsed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chatml_template() {
        let result = build_chatml_prompt(Some("Be concise"), "Hello");
        assert!(result.contains("Be concise"));
        assert!(result.contains("Hello"));
        assert!(result.contains("<|im_start|>assistant"));
    }

    #[test]
    fn test_extract_completion_clean() {
        let text = "Generated response\n> EOF by user\n\n";
        assert_eq!(extract_completion_output(text), "Generated response");
    }

    #[test]
    fn test_extract_completion_no_eof() {
        assert_eq!(extract_completion_output("Hello world"), "Hello world");
    }

    #[test]
    fn test_extract_completion_empty() {
        assert_eq!(extract_completion_output(""), "");
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate_to_tokens("hello", 100), "hello");
    }

    #[test]
    fn test_truncate_long() {
        let result = truncate_to_tokens("hello world", 5);
        assert!(result.len() <= 8);
        assert!(result.ends_with("..."));
    }
}
