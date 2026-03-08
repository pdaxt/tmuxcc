# DX Terminal

**The AI-native terminal multiplexer.** Monitor, manage, and orchestrate AI coding agents from one screen.

Open source. No login. No telemetry.

```
╭─────────────────────────────────────────────────────────────────────────────────╮
│ DX │ 12 agents │ ⠹ 3 working │ ready │ ▄▆▅▅▅▆▅ 42% │ MEM 8.2G/16G │ 14:30  │
╰─────────────────────────────────────────────────────────────────────────────────╯
╭ 12 agents ──────────────────────╮╭ Preview ──────────────────────────────────────╮
│▼ session-1                      ││ Claude Code wants to edit:                    │
│ ├─ 1: backend                   ││ src/main.rs                                   │
│   ├─● /api-server               ││                                               │
│   │  Claude │ Idle │ 12m        ││ - fn main() {                                 │
│   ├─⠹ /api-server               ││ + fn main() -> Result<()> {                   │
│   │  Claude │ Working │ 3m      ││                                               │
│   └─● /shared                   ││ Allow? [y/n]                                  │
│      Gemini │ Idle │ 45m        ││                                               │
│▼ session-2                      │╰───────────────────────────────────────────────╯
│ ├─ 0: frontend                  │╭ Input ─────────────────────────────────────────╮
│   ├─⚠ /web-app                  ││ > fix the auth bug in login.tsx                │
│   │  Claude │ APPROVAL [Edit]   │╰───────────────────────────────────────────────╯
╰─────────────────────────────────╯╭ Token Usage ──────────────────────────────────╮
│ Session: 45K in / 12K out │ Cost: $0.42 │ Today: $3.80                           │
╰──────────────────────────────────────────────────────────────────────────────────╯
```

## Install

**Homebrew (macOS & Linux):**
```bash
brew install pdaxt/tap/dx-terminal
```

**Cargo:**
```bash
cargo install dx-terminal
```

**Shell script:**
```bash
curl -fsSL https://raw.githubusercontent.com/pdaxt/dx-terminal/main/install.sh | bash
```

**From source:**
```bash
git clone https://github.com/pdaxt/dx-terminal.git
cd dx-terminal
cargo install --path .
```

## Usage

```bash
dx                    # Launch (native PTY mode)
dx --tmux             # Legacy tmux monitoring mode
dx --debug            # Write debug logs
dx --init-config      # Generate config file
dx --show-config-path # Show config location
```

## What it does

DX Terminal detects and monitors AI coding agents running in your terminal:

| Agent | Detected |
|-------|----------|
| Claude Code | Yes |
| OpenCode | Yes |
| Codex CLI | Yes |
| Gemini CLI | Yes |

For each agent it shows:
- **Status** — idle, processing, awaiting approval, error
- **Subagents** — tracks spawned sub-tasks with their lifecycle
- **Context** — remaining context window percentage
- **Cost** — token usage and estimated cost (per session, per day, per project)
- **Git** — current branch, uncommitted changes
- **Working directory** — abbreviated path

## Key Bindings

| Key | Action |
|-----|--------|
| `j`/`k` or arrows | Navigate agents |
| `y` | Approve pending request |
| `n` | Reject pending request |
| `a` | Approve ALL pending |
| `1`-`9` | Answer numbered choices |
| `Space` | Toggle selection |
| `f` | Focus (jump to agent's pane) |
| `i` | Input mode (type to agent) |
| `X` | Toggle analytics panel |
| `s` | Toggle subagent log |
| `q` | Quit |

## Architecture

Built entirely in Rust. Native PTY management — no tmux dependency.

- **PTY Manager** — `portable-pty` for spawning and managing terminal sessions
- **VTE Parser** — `vte` crate for real-time terminal output parsing
- **Agent Detection** — process tree inspection + output pattern matching
- **Analytics** — SQLite-backed token/cost tracking with per-project breakdown
- **TUI** — `ratatui` + `crossterm` for the interface
- **MCP Server** — built-in MCP so other AI agents can control this terminal

## Configuration

```bash
dx --init-config      # Creates default config
dx --show-config-path # Shows path
```

Config file (TOML):
```toml
poll_interval_ms = 500
capture_lines = 100
api_url = "http://localhost:3100"

[[agent_patterns]]
pattern = "my-custom-agent"
agent_type = "CustomAgent"
```

| OS | Config Path |
|----|-------------|
| macOS | `~/Library/Application Support/dx-terminal/config.toml` |
| Linux | `~/.config/dx-terminal/config.toml` |

## License

MIT

## Contributing

```bash
git clone https://github.com/pdaxt/dx-terminal.git
cd dx-terminal
cargo test
cargo clippy
cargo fmt
```

PRs welcome. Run the checks above before submitting.
