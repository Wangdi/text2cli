# text2cli

AI-powered command suggestion CLI that integrates with your shell.

## Installation

```bash
cargo install --path .
```

## Shell Integration

### PowerShell

Add to your `$PROFILE`:

```powershell
. /path/to/shell/text2cli.ps1
```

Or use the init command:

```powershell
Invoke-Expression (text2cli init pwsh)
```

### Bash

Add to `~/.bashrc`:

```bash
source /path/to/shell/text2cli.bash
```

Or:

```bash
eval "$(text2cli init bash)"
```

### Zsh

Add to `~/.zshrc`:

```zsh
source /path/to/shell/text2cli.zsh
```

Or:

```zsh
eval "$(text2cli init zsh)"
```

## Usage

Type `@@@` followed by your instruction:

```
$ @@@ 重命名这个变量
$ git mv old_name new_name  # Command injected, press Enter to execute
```

## Configuration

Configuration file: `~/.text2cli/config.toml`

```toml
trigger = "@@@"
default_agent = "claude-code"

[agents.claude-code]
enabled = true
command = "claude"

[agents.codex]
enabled = true
command = "codex"
```

## Supported Agents

- claude-code (Anthropic)
- codex (OpenAI)
- opencode
- cursor-cli
- gemini
- And more via generic adapter

## License

MIT
