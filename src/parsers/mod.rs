mod claude_code;
mod codex_cli;
mod gemini_cli;
mod opencode;

pub use claude_code::ClaudeCodeParser;
pub use codex_cli::CodexCliParser;
pub use gemini_cli::GeminiCliParser;
pub use opencode::OpenCodeParser;

use crate::agents::{AgentStatus, AgentType, Subagent};
use crate::tmux::PaneInfo;

/// Safely get the last N characters of a string (handles multi-byte chars)
pub(crate) fn safe_tail(s: &str, max_chars: usize) -> &str {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s;
    }
    let skip = char_count - max_chars;
    let byte_idx = s.char_indices().nth(skip).map(|(idx, _)| idx).unwrap_or(0);
    &s[byte_idx..]
}

/// Trait for parsing agent output
pub trait AgentParser: Send + Sync {
    /// Returns the name of the agent
    fn agent_name(&self) -> &str;

    /// Returns the AgentType for this parser
    fn agent_type(&self) -> AgentType;

    /// Checks if any of the detection strings match this agent
    fn matches(&self, detection_strings: &[&str]) -> bool;

    /// Parses the pane content and returns the agent status
    fn parse_status(&self, content: &str) -> AgentStatus;

    /// Parses subagents from the content (default: empty)
    fn parse_subagents(&self, content: &str) -> Vec<Subagent> {
        let _ = content;
        Vec::new()
    }

    /// Parses context remaining percentage from content (default: None)
    fn parse_context_remaining(&self, content: &str) -> Option<u8> {
        let _ = content;
        None
    }

    /// Returns the key(s) to send for approval
    fn approval_keys(&self) -> &str {
        "y"
    }

    /// Returns the key(s) to send for rejection
    fn rejection_keys(&self) -> &str {
        "n"
    }
}

/// Registry of all available parsers
pub struct ParserRegistry {
    parsers: Vec<Box<dyn AgentParser>>,
}

impl ParserRegistry {
    /// Creates a new registry with all default parsers
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(ClaudeCodeParser::new()),
                Box::new(OpenCodeParser::new()),
                Box::new(CodexCliParser::new()),
                Box::new(GeminiCliParser::new()),
            ],
        }
    }

    /// Finds a parser that matches the given pane info
    pub fn find_parser_for_pane(&self, pane: &PaneInfo) -> Option<&dyn AgentParser> {
        let detection_strings = pane.detection_strings();
        self.parsers
            .iter()
            .find(|p| p.matches(&detection_strings))
            .map(|p| p.as_ref())
    }

}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_registry() {
        let registry = ParserRegistry::new();

        // Test finding parsers with various detection strings
        let claude_pane = PaneInfo {
            session: "main".to_string(),
            window: 0,
            window_name: "code".to_string(),
            pane: 0,
            command: "node".to_string(),
            title: "Claude Code".to_string(),
            path: "/home/user/project".to_string(),
            pid: 1234,
            cmdline: "/usr/bin/claude".to_string(),
            child_commands: Vec::new(),
        };
        assert!(registry.find_parser_for_pane(&claude_pane).is_some());

        let opencode_pane = PaneInfo {
            session: "main".to_string(),
            window: 0,
            window_name: "code".to_string(),
            pane: 1,
            command: "opencode".to_string(),
            title: "".to_string(),
            path: "/home/user/project".to_string(),
            pid: 1235,
            cmdline: "opencode".to_string(),
            child_commands: Vec::new(),
        };
        assert!(registry.find_parser_for_pane(&opencode_pane).is_some());

        // Test detection via child processes
        let child_claude_pane = PaneInfo {
            session: "main".to_string(),
            window: 0,
            window_name: "code".to_string(),
            pane: 2,
            command: "zsh".to_string(),
            title: "~".to_string(),
            path: "/home/user/project".to_string(),
            pid: 1236,
            cmdline: "-zsh".to_string(),
            child_commands: vec!["claude -c".to_string(), "claude".to_string()],
        };
        assert!(registry.find_parser_for_pane(&child_claude_pane).is_some());
    }
}
