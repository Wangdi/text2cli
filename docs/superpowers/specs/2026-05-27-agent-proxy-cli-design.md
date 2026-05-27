# Agent Proxy CLI Design

## Overview

A cross-platform CLI tool that integrates with shell environments to provide AI-powered command suggestions. When users input commands prefixed with `@@@`, the tool invokes a configured coding agent CLI, receives suggested commands, and injects them into the terminal input buffer for user confirmation.

## Core Workflow

```
User input: @@@ 重命名这个变量 <Enter>
    ↓
agent-proxy detects trigger
    ↓
Calls configured agent CLI
    ↓
Agent returns suggested command: git mv old_name new_name
    ↓
Injects to input buffer: $> git mv old_name new_name█
    ↓
User presses Enter to execute, or modifies first
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Terminal (bash/zsh/pwsh)             │
│  ┌─────────────────────────────────────────────────────┐│
│  │              Shell + agent-proxy hook               ││
│  │  User input: @@@ 重命名这个变量 <Enter>             ││
│  │      ↓                                              ││
│  │  Hook intercepts → Detects trigger → Calls agent    ││
│  │      ↓                                              ││
│  │  Agent returns: git mv old_name new_name           ││
│  │      ↓                                              ││
│  │  Inject to buffer: $> git mv old_name new_name█     ││
│  │      ↓                                              ││
│  │  User presses Enter or modifies                     ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

## Core Components

| Component | Responsibility | Key Interface |
|-----------|----------------|---------------|
| **TriggerParser** | Parse input, detect `@@@` trigger | `parse(input) -> Option<Command>` |
| **ContextCollector** | Collect execution context | `collect() -> Context` |
| **AgentRouter** | Select agent based on config | `route(config) -> AgentExecutor` |
| **AgentExecutor** | Execute agent CLI, handle I/O | `execute(cmd, context) -> Result` |
| **SessionManager** | Manage session state (optional) | `create()/resume()/end()` |
| **ConfigLoader** | Load configuration file | `load() -> Config` |

## Data Structures

```rust
struct Command {
    trigger: String,        // "@@@"
    content: String,        // User instruction content
    options: CommandOptions, // --session, etc.
}

struct Context {
    working_dir: PathBuf,
    git_branch: Option<String>,
    git_status: Option<GitStatus>,
    recent_files: Vec<PathBuf>,
    shell_env: HashMap<String, String>,
}

struct Config {
    trigger: String,
    default_agent: String,
    agents: HashMap<String, AgentConfig>,
}
```

## Shell Integration

### Linux/macOS (bash)

```bash
# ~/.bashrc
eval "$(agent-proxy init bash)"
```

Implementation:
- Use `preexec` trap to intercept commands
- Detect `@@@` prefix
- Call agent-proxy to get command
- Use `READLINE_LINE` + `bind -x` or expect for buffer injection

### Linux/macOS (zsh)

```zsh
# ~/.zshrc
eval "$(agent-proxy init zsh)"
```

Implementation:
- Use `preexec` function to intercept commands
- Detect `@@@` prefix
- Call agent-proxy to get command
- Use `print -z "command"` to inject into input buffer

### Windows (PowerShell)

```powershell
# $PROFILE
Invoke-Expression (agent-proxy init pwsh)
```

Implementation:
- Use `PSReadLine` handlers
- Custom key handler or `AddToHistoryHandler`

### Command Injection Methods

| Platform | Method |
|----------|--------|
| zsh | `print -z "command"` |
| bash | `READLINE_LINE` + `bind -x` or expect |
| pwsh | `[PSConsoleUtilities]::PSReadLine::Insert('command')` |

## Configuration

Default location: `~/.agent-proxy/config.toml`

```toml
trigger = "@@@"
default_agent = "claude-code"

[agents.claude-code]
enabled = true
command = "claude"

[agents.codex]
enabled = true
command = "codex"

[agents.opencode]
enabled = false
command = "opencode"

[agents.cursor-cli]
enabled = false
command = "cursor-cli"

[agents.gemini]
enabled = false
command = "gemini"

[agents.openclaw]
enabled = false
command = "openclaw"

[agents.hermes]
enabled = false
command = "hermes"
```

## Session Mode

Default: Single execution (no context persistence)

With `--session`:
```bash
$ agent-proxy --session my-project
> @@@ 重命名这个变量
[Processing...]
> @@@ 把它改成 camelCase
[Processing...]
> @@@ exit
Session ended
```

## Agent Protocol

### Prompt Construction

agent-proxy constructs a structured prompt for each agent to ensure consistent output:

```
You are a command suggestion assistant. Given the user's request and context,
respond with ONLY the shell command(s) to execute. No explanations.

Context:
- Working directory: /path/to/project
- Git branch: main
- Recent files: src/main.rs, src/lib.rs

User request: 重命名这个变量

Respond with the command only, one per line if multiple.
```

### Output Parsing

Agent adapters extract commands from agent output:

| Agent | Expected Output | Parsing Strategy |
|-------|-----------------|------------------|
| claude-code | Plain command or code block | Extract from ```bash blocks or plain text |
| codex | JSON or plain text | Parse JSON `command` field or extract first line |
| opencode | Plain text | Extract command from response |
| generic | Plain text | First non-empty line, strip markdown if present |

### AgentAdapter Trait

```rust
trait AgentAdapter {
    /// Build prompt with context
    fn build_prompt(&self, request: &str, context: &Context) -> String;
    
    /// Parse agent output into command(s)
    fn parse_output(&self, output: &str) -> Result<Vec<String>>;
    
    /// Get CLI command to invoke
    fn command(&self) -> &str;
}
```

## Error Handling

| Scenario | Handling |
|----------|----------|
| Agent CLI not installed | Error message + installation instructions |
| Agent returns empty/invalid | "Agent did not return valid command", keep original input |
| Network timeout | Timeout message, allow retry |
| Config file missing | Use defaults, hint user can create config |
| Trigger parse failure | Pass through as normal command |

Error output format:
```
[agent-proxy] Error: Agent 'claude-code' not found
[agent-proxy] Install with: npm install -g @anthropic/claude-code
```

## Testing Strategy

| Test Type | Coverage |
|-----------|----------|
| **Unit tests** | TriggerParser, ContextCollector, ConfigLoader |
| **Integration tests** | AgentRouter interaction with agent CLIs |
| **Shell integration tests** | Hook behavior in mock shell environment |
| **E2E tests** | Full flow: input → agent → inject → confirm |

Test frameworks:
- Rust `#[test]` for unit tests
- `assert_cmd` for CLI testing
- `bats-core` for bash script testing

## Project Structure

```
agent-proxy/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── parser.rs            # TriggerParser
│   ├── context.rs           # ContextCollector
│   ├── router.rs            # AgentRouter
│   ├── executor.rs          # AgentExecutor
│   ├── session.rs           # SessionManager
│   ├── config.rs            # ConfigLoader
│   ├── shell/
│   │   ├── mod.rs
│   │   ├── bash.rs          # bash hook generation
│   │   ├── zsh.rs           # zsh hook generation
│   │   └── pwsh.rs          # PowerShell hook generation
│   └── agents/
│       ├── mod.rs
│       ├── claude.rs        # Claude Code adapter
│       ├── codex.rs         # Codex adapter
│       └── generic.rs       # Generic agent adapter
├── tests/
│   ├── integration_tests.rs
│   └── fixtures/
├── shell/
│   ├── agent-proxy.bash     # bash hook script
│   ├── agent-proxy.zsh      # zsh hook script
│   └── agent-proxy.ps1      # PowerShell hook script
└── config.example.toml
```

## CLI Commands

```
agent-proxy init <shell>      # Generate shell hook script
agent-proxy <command>         # Execute with command
agent-proxy --session <name>  # Start/resume session
agent-proxy --agent <name>    # Use specific agent
agent-proxy config            # Show/edit config
agent-proxy list-agents       # List available agents
```

## Success Criteria

1. Works in bash, zsh, and PowerShell on Windows, Linux, and macOS
2. Correctly detects `@@@` trigger and invokes configured agent
3. Successfully injects suggested commands into terminal input buffer
4. User can confirm, modify, or cancel suggested commands
5. Configuration supports multiple agents with enable/disable
6. Session mode maintains context across multiple commands
7. Clear error messages with actionable guidance
