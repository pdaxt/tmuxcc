<p align="center">
  <h1 align="center">This project has moved to DX Terminal</h1>
</p>

<p align="center">
  <strong>AgentOS has been rebranded and merged into <a href="https://github.com/pdaxt/dx-terminal">DX Terminal</a>.</strong>
</p>

---

## DX Terminal

The AI-native terminal multiplexer. Monitor, manage, and orchestrate AI coding agents from one screen.

**Go to the new repo:** [github.com/pdaxt/dx-terminal](https://github.com/pdaxt/dx-terminal)

### What changed?

AgentOS and TmuxCC were merged into a single product: **DX Terminal** (`dx`).

- Binary: `dx` (was `agentos`)
- Package: `dx-terminal` (was `agentos`)
- Native PTY mode (no tmux dependency)
- Real-time token cost tracking
- Context window gauges
- 30k+ lines of Rust, 9.5MB single binary

### Quick start

```bash
git clone https://github.com/pdaxt/dx-terminal.git
cd dx-terminal
cargo build --release
./target/release/dx
```

---

This repo is archived. All development continues at [pdaxt/dx-terminal](https://github.com/pdaxt/dx-terminal).
