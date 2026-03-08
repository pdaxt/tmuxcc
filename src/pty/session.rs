//! A single PTY session — one process (e.g. claude-start) in one pane.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use parking_lot::Mutex;
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use tokio::sync::mpsc;
use vte::{Params, Parser, Perform};

/// Events emitted by a PTY session
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// New output bytes (raw terminal data)
    Output { session_id: String, data: Vec<u8> },
    /// Process exited
    Exited { session_id: String, exit_code: Option<u32> },
    /// Token usage detected in output
    TokenUsage {
        session_id: String,
        input_tokens: u64,
        output_tokens: u64,
        cache_read: u64,
    },
}

/// Handle for interacting with a running PTY session
#[derive(Clone)]
pub struct PtySessionHandle {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub session_id: String,
}

impl PtySessionHandle {
    /// Write raw bytes to the PTY (for keyboard input)
    pub fn write_bytes(&self, data: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock();
        writer.write_all(data)?;
        writer.flush()?;
        Ok(())
    }

    /// Write a string followed by Enter
    pub fn write_line(&self, line: &str) -> Result<()> {
        let mut writer = self.writer.lock();
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\r")?;
        writer.flush()?;
        Ok(())
    }

    /// Send bracketed paste (proper paste protocol)
    pub fn paste(&self, text: &str) -> Result<()> {
        let mut writer = self.writer.lock();
        // Bracketed paste mode: \x1b[200~ ... \x1b[201~
        writer.write_all(b"\x1b[200~")?;
        writer.write_all(text.as_bytes())?;
        writer.write_all(b"\x1b[201~")?;
        writer.flush()?;
        Ok(())
    }

    /// Send Ctrl+C
    pub fn interrupt(&self) -> Result<()> {
        self.write_bytes(&[0x03])
    }

    /// Resize the PTY
    pub fn resize(&self, _rows: u16, _cols: u16) -> Result<()> {
        // Note: resize needs the master_pty reference, handled by PtySession
        Ok(())
    }
}

/// Scrollback buffer that stores decoded terminal output
#[derive(Debug)]
pub struct ScrollbackBuffer {
    /// Raw lines of text (stripped of escape sequences)
    lines: Vec<String>,
    /// Current line being built
    current_line: String,
    /// Max lines to keep
    max_lines: usize,
}

impl ScrollbackBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            current_line: String::new(),
            max_lines,
        }
    }

    /// Get the last N lines as a single string
    pub fn tail(&self, n: usize) -> String {
        let start = self.lines.len().saturating_sub(n);
        let mut result: String = self.lines[start..]
            .iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if !self.current_line.is_empty() {
            result.push('\n');
            result.push_str(&self.current_line);
        }
        result
    }

    /// Get total line count
    pub fn line_count(&self) -> usize {
        self.lines.len() + if self.current_line.is_empty() { 0 } else { 1 }
    }
}

/// VTE performer that builds scrollback from escape-sequence-stripped output
struct ScrollbackPerformer<'a> {
    buffer: &'a mut ScrollbackBuffer,
}

impl<'a> Perform for ScrollbackPerformer<'a> {
    fn print(&mut self, c: char) {
        self.buffer.current_line.push(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // Newline
            0x0A => {
                let line = std::mem::take(&mut self.buffer.current_line);
                self.buffer.lines.push(line);
                if self.buffer.lines.len() > self.buffer.max_lines {
                    self.buffer.lines.remove(0);
                }
            }
            // Carriage return
            0x0D => {
                self.buffer.current_line.clear();
            }
            // Tab
            0x09 => {
                self.buffer.current_line.push('\t');
            }
            // Backspace
            0x08 => {
                self.buffer.current_line.pop();
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn csi_dispatch(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

/// A managed PTY session
pub struct PtySession {
    pub id: String,
    pub project: String,
    pub project_path: PathBuf,
    pub role: String,
    pub pane_num: u8,
    pub theme: String,
    pub started_at: Instant,
    pub scrollback: Arc<Mutex<ScrollbackBuffer>>,
    child: Box<dyn Child + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    event_tx: mpsc::Sender<SessionEvent>,
}

impl PtySession {
    /// Spawn a new PTY session running the given command
    pub fn spawn(
        id: String,
        project: String,
        project_path: PathBuf,
        role: String,
        pane_num: u8,
        theme: String,
        command: &str,
        args: &[&str],
        env: Vec<(String, String)>,
        rows: u16,
        cols: u16,
        event_tx: mpsc::Sender<SessionEvent>,
    ) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open PTY")?;

        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);
        cmd.cwd(&project_path);

        // Set environment
        for (key, value) in &env {
            cmd.env(key, value);
        }
        // Always set TERM
        cmd.env("TERM", "xterm-256color");

        let child = pair.slave.spawn_command(cmd).context("Failed to spawn process")?;

        let scrollback = Arc::new(Mutex::new(ScrollbackBuffer::new(10000)));

        Ok(Self {
            id,
            project,
            project_path,
            role,
            pane_num,
            theme,
            started_at: Instant::now(),
            scrollback,
            child,
            master: pair.master,
            event_tx,
        })
    }

    /// Get a handle for writing to this session
    pub fn handle(&self) -> Result<PtySessionHandle> {
        let writer = self.master.take_writer()?;
        Ok(PtySessionHandle {
            writer: Arc::new(Mutex::new(writer)),
            session_id: self.id.clone(),
        })
    }

    /// Start the output reader loop (run in a tokio task)
    pub fn start_reader(self) -> Result<(PtySessionHandle, tokio::task::JoinHandle<()>)> {
        let handle = self.handle()?;
        let scrollback = self.scrollback.clone();
        let event_tx = self.event_tx.clone();
        let session_id = self.id.clone();

        let mut reader = self.master.try_clone_reader()?;

        let join_handle = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            let mut parser = Parser::new();

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = buf[..n].to_vec();

                        // Parse through VTE to build scrollback
                        {
                            let mut sb = scrollback.lock();
                            let mut performer = ScrollbackPerformer { buffer: &mut sb };
                            for byte in &data {
                                parser.advance(&mut performer, *byte);
                            }
                        }

                        // Check for token usage patterns in the raw text
                        let text = String::from_utf8_lossy(&data);
                        if let Some(usage) = parse_token_usage(&text, &session_id) {
                            let _ = event_tx.blocking_send(usage);
                        }

                        // Send raw output event
                        let _ = event_tx.blocking_send(SessionEvent::Output {
                            session_id: session_id.clone(),
                            data,
                        });
                    }
                    Err(e) => {
                        tracing::debug!("PTY read error for {}: {}", session_id, e);
                        break;
                    }
                }
            }

            // Process exited
            let _ = event_tx.blocking_send(SessionEvent::Exited {
                session_id,
                exit_code: None,
            });
        });

        Ok((handle, join_handle))
    }

    /// Resize the PTY
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    /// Check if the child process is still running
    pub fn is_running(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }
}

/// Parse Claude Code token usage from output text.
/// Looks for patterns like: "Input: 1234 | Output: 5678 | Cache read: 9012"
fn parse_token_usage(text: &str, session_id: &str) -> Option<SessionEvent> {
    // Claude Code shows token stats in various formats:
    // "tokens: 1.2k input, 3.4k output"
    // "Input tokens: 1234 | Output tokens: 5678"
    // Context remaining patterns
    let input_re = regex::Regex::new(r"(?i)input[:\s]+tokens?[:\s]+(\d[\d,.k]*)|(\d[\d,.k]*)\s*input").ok()?;
    let output_re = regex::Regex::new(r"(?i)output[:\s]+tokens?[:\s]+(\d[\d,.k]*)|(\d[\d,.k]*)\s*output").ok()?;
    let cache_re = regex::Regex::new(r"(?i)cache[:\s]+read[:\s]+(\d[\d,.k]*)").ok()?;

    let parse_num = |s: &str| -> u64 {
        let s = s.replace(',', "");
        if s.ends_with('k') || s.ends_with('K') {
            let n: f64 = s.trim_end_matches(['k', 'K']).parse().unwrap_or(0.0);
            (n * 1000.0) as u64
        } else {
            s.parse().unwrap_or(0)
        }
    };

    let input_tokens = input_re.captures(text)
        .and_then(|c| c.get(1).or(c.get(2)))
        .map(|m| parse_num(m.as_str()))
        .unwrap_or(0);

    let output_tokens = output_re.captures(text)
        .and_then(|c| c.get(1).or(c.get(2)))
        .map(|m| parse_num(m.as_str()))
        .unwrap_or(0);

    let cache_read = cache_re.captures(text)
        .and_then(|c| c.get(1))
        .map(|m| parse_num(m.as_str()))
        .unwrap_or(0);

    if input_tokens > 0 || output_tokens > 0 {
        Some(SessionEvent::TokenUsage {
            session_id: session_id.to_string(),
            input_tokens,
            output_tokens,
            cache_read,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollback_buffer() {
        let mut buf = ScrollbackBuffer::new(5);
        buf.current_line = "hello".to_string();
        buf.lines.push("line 1".to_string());
        buf.lines.push("line 2".to_string());

        assert_eq!(buf.line_count(), 3);
        let tail = buf.tail(10);
        assert!(tail.contains("line 1"));
        assert!(tail.contains("hello"));
    }

    #[test]
    fn test_parse_token_usage() {
        let text = "Input tokens: 1234 | Output tokens: 5678";
        let event = parse_token_usage(text, "test");
        assert!(event.is_some());
        if let Some(SessionEvent::TokenUsage { input_tokens, output_tokens, .. }) = event {
            assert_eq!(input_tokens, 1234);
            assert_eq!(output_tokens, 5678);
        }
    }

    #[test]
    fn test_parse_token_usage_k_format() {
        let text = "tokens: 1.2k input, 3.4k output";
        let event = parse_token_usage(text, "test");
        assert!(event.is_some());
        if let Some(SessionEvent::TokenUsage { input_tokens, output_tokens, .. }) = event {
            assert_eq!(input_tokens, 1200);
            assert_eq!(output_tokens, 3400);
        }
    }
}
