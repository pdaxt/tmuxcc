use regex::Regex;

use crate::agents::{AgentStatus, AgentType, ApprovalType, Subagent};

use super::{safe_tail, AgentParser};

/// Parser for Gemini CLI output
pub struct GeminiCliParser {
    approval_pattern: Regex,
    processing_pattern: Regex,
    idle_pattern: Regex,
}

impl GeminiCliParser {
    pub fn new() -> Self {
        Self {
            approval_pattern: Regex::new(r"(?i)\[y/n\]|\[yes/no\]|confirm|approve|allow").expect("invalid regex"),
            processing_pattern: Regex::new(r"(?i)(thinking|generating|processing|analyzing)")
                .expect("invalid regex"),
            idle_pattern: Regex::new(r"(?i)(ready|waiting|>\s*$)").expect("invalid regex"),
        }
    }
}

impl Default for GeminiCliParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentParser for GeminiCliParser {
    fn agent_name(&self) -> &str {
        "Gemini CLI"
    }

    fn agent_type(&self) -> AgentType {
        AgentType::GeminiCli
    }

    fn matches(&self, detection_strings: &[&str]) -> bool {
        detection_strings.iter().any(|s| {
            let lower = s.to_lowercase();
            lower.contains("gemini") || lower.contains("google")
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
                activity: "Generating...".to_string(),
            };
        }

        if self.idle_pattern.is_match(recent) {
            return AgentStatus::Idle;
        }

        AgentStatus::Unknown
    }

    fn parse_subagents(&self, _content: &str) -> Vec<Subagent> {
        // Gemini CLI doesn't have subagents
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let parser = GeminiCliParser::new();
        assert!(parser.matches(&["gemini", "", ""]));
        assert!(parser.matches(&["", "Gemini CLI", ""]));
        assert!(!parser.matches(&["claude", "Claude", ""]));
    }
}
