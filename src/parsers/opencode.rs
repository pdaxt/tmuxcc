use regex::Regex;

use crate::agents::{AgentStatus, AgentType, ApprovalType, Subagent};

use super::{safe_tail, AgentParser};

/// Parser for OpenCode CLI output
pub struct OpenCodeParser {
    approval_pattern: Regex,
    processing_pattern: Regex,
    idle_pattern: Regex,
}

impl OpenCodeParser {
    pub fn new() -> Self {
        Self {
            approval_pattern: Regex::new(r"(?i)\[y/n\]|\[yes/no\]|confirm|approve|allow").unwrap(),
            processing_pattern: Regex::new(
                r"(?i)(thinking|processing|generating|analyzing|working)",
            )
            .unwrap(),
            idle_pattern: Regex::new(r"(?i)(ready|waiting|idle|>\s*$)").unwrap(),
        }
    }
}

impl Default for OpenCodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentParser for OpenCodeParser {
    fn agent_name(&self) -> &str {
        "OpenCode"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::OpenCode
    }

    fn matches(&self, detection_strings: &[&str]) -> bool {
        detection_strings.iter().any(|s| {
            let lower = s.to_lowercase();
            lower.contains("opencode") || lower.contains("open-code")
        })
    }

    fn parse_status(&self, content: &str) -> AgentStatus {
        let recent = safe_tail(content, 500);

        if self.approval_pattern.is_match(recent) {
            return AgentStatus::AwaitingApproval {
                approval_type: ApprovalType::Other("Pending".to_string()),
                details: String::new(),
            };
        }

        if self.processing_pattern.is_match(recent) {
            return AgentStatus::Processing {
                activity: "Processing...".to_string(),
            };
        }

        if self.idle_pattern.is_match(recent) {
            return AgentStatus::Idle;
        }

        AgentStatus::Unknown
    }

    fn parse_subagents(&self, _content: &str) -> Vec<Subagent> {
        // OpenCode doesn't have subagents
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let parser = OpenCodeParser::new();
        assert!(parser.matches(&["opencode", "", ""]));
        assert!(parser.matches(&["", "OpenCode", ""]));
        assert!(!parser.matches(&["claude", "Claude Code", ""]));
    }

    #[test]
    fn test_parse_processing() {
        let parser = OpenCodeParser::new();
        let content = "Thinking about your request...";
        let status = parser.parse_status(content);

        assert!(matches!(status, AgentStatus::Processing { .. }));
    }
}
