//! The three text operations: translate, correct and refine.

use crate::history;
use crate::llm::{self, LlmConfig};
use crate::settings::Settings;

pub const DEFAULT_TRANSLATE_PROMPT: &str = "You are an expert translator. Translate the following text from {original_lang} to {target_lang}. Provide only the translated text, without any additional explanations or formatting.";
pub const DEFAULT_CORRECT_PROMPT: &str = "You are an expert in grammar. Correct the grammar of the following text in {target_lang}. Provide only the corrected text, without any additional explanations or formatting.";
pub const DEFAULT_REFINE_PROMPT: &str = "You are an expert editor. Rewrite the following text in a more conversational style, in {target_lang}. Output a single rewrite only — no options, no alternatives, no explanations, no labels, no formatting.";

/// Source language meaning "let the model detect it". Stored as-is in
/// history; spelled out only inside the prompt.
pub const AUTO_LANG: &str = "auto";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Translate,
    Correct,
    Refine,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Translate, Mode::Correct, Mode::Refine];

    pub fn label(self) -> &'static str {
        match self {
            Mode::Translate => "Translate",
            Mode::Correct => "Correct",
            Mode::Refine => "Refine",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Mode::Translate => 0,
            Mode::Correct => 1,
            Mode::Refine => 2,
        }
    }

    /// Key stored in history entries.
    pub fn key(self) -> &'static str {
        match self {
            Mode::Translate => "translate",
            Mode::Correct => "correct",
            Mode::Refine => "refine",
        }
    }

    pub fn default_prompt(self) -> &'static str {
        match self {
            Mode::Translate => DEFAULT_TRANSLATE_PROMPT,
            Mode::Correct => DEFAULT_CORRECT_PROMPT,
            Mode::Refine => DEFAULT_REFINE_PROMPT,
        }
    }

    fn custom_prompt(self, s: &Settings) -> &str {
        match self {
            Mode::Translate => &s.prompt_translate,
            Mode::Correct => &s.prompt_correct,
            Mode::Refine => &s.prompt_refine,
        }
    }

    fn text_label(self) -> &'static str {
        match self {
            Mode::Translate => "Text to translate",
            Mode::Correct => "Text to correct",
            Mode::Refine => "Text to refine",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub mode: Mode,
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

/// Builds the full prompt for a request.
pub fn build_prompt(settings: &Settings, req: &Request) -> String {
    let custom = req.mode.custom_prompt(settings).trim();
    let template = if custom.is_empty() {
        req.mode.default_prompt()
    } else {
        custom
    };
    let source = if req.source_lang == AUTO_LANG {
        "the language of the text (detect it)"
    } else {
        &req.source_lang
    };
    let instructions = template
        .replace("{original_lang}", source)
        .replace("{target_lang}", &req.target_lang);
    format!(
        "<start_of_turn>user\n{instructions}\n\n{}: {}\n<end_of_turn>\n<start_of_turn>model",
        req.mode.text_label(),
        req.text
    )
}

/// Runs a request against the configured provider (blocking) and records it
/// in history when enabled.
pub fn run(settings: &Settings, req: &Request) -> Result<String, String> {
    let prompt = build_prompt(settings, req);
    let output = llm::complete(&LlmConfig::from_settings(settings), &prompt)?;
    if settings.history_enabled {
        history::append(req.mode.key(), &req.text, &req.source_lang, &req.target_lang);
    }
    Ok(llm::trim_thinking_blocks(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_uses_override_and_languages() {
        let mut s = Settings::default();
        let req = Request {
            mode: Mode::Translate,
            text: "hello".into(),
            source_lang: "English".into(),
            target_lang: "Français".into(),
        };
        let p = build_prompt(&s, &req);
        assert!(p.contains("from English to Français"));
        assert!(p.contains("Text to translate: hello"));

        let auto = Request {
            source_lang: AUTO_LANG.into(),
            ..req.clone()
        };
        assert!(build_prompt(&s, &auto).contains("from the language of the text (detect it) to"));

        s.prompt_translate = "Say {target_lang}".into();
        assert!(build_prompt(&s, &req).contains("Say Français"));
    }
}
