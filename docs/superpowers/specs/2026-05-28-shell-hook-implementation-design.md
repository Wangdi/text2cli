# Shell Hook Implementation Design

## Overview

This design improves the shell hook implementation for text2cli to enable proper command injection for Bash, Zsh, and PowerShell.

## Problem Statement

The current implementation has fundamental issues:
1. **Bash**: Uses `preexec_functions` but tries to set `READLINE_LINE`, which only works in `bind -x` widget context
2. **Zsh**: Uses `preexec_functions` with `print -z`, but preexec runs after Enter is pressed, so buffer injection is too late
3. **PowerShell**: PSReadLine integration is partially correct but has implementation issues

## Solution: Enter Key Interception

All three shells will intercept the Enter key to check for the trigger pattern and transform commands before execution.

## Shell-Specific Designs

### 1. Bash Hook

#### Architecture

```
User presses Enter
       ↓
Custom widget invoked (via bind -x)
       ↓
Check READLINE_LINE for trigger (@@@)
       ↓
┌─────────────────────────────────────┐
│ Trigger found?                      │
│   Yes → Transform command           │
│        → Replace READLINE_LINE      │
│        → Call accept-line           │
│   No  → Call original accept-line   │
└─────────────────────────────────────┘
```

#### Implementation Details

1. **Widget Function**: Create a bash function that:
   - Checks if `READLINE_LINE` starts with `@@@`
   - If yes: runs `{binary} process "$READLINE_LINE"`, sets result to `READLINE_LINE`
   - Calls `builtin accept-line` or uses `READLINE_MARK` for execution

2. **Binding Setup**:
   - Use `bind -x '"\C-M": __text2cli_accept_line'` to bind Enter (Ctrl+M)
   - Alternative: Use `bind '"\C-M": "\C-x\C-t"'` with `bind -x '"\C-x\C-t": __text2cli_accept_line'`

3. **Error Handling**:
   - Check for empty results
   - Handle transformation errors gracefully
   - Preserve original behavior if transformation fails

#### Generated Script Structure

```bash
# text2cli bash integration
# Add to ~/.bashrc

__text2cli_accept_line() {
    local cmd="$READLINE_LINE"
    if [[ "$cmd" == @@@* ]]; then
        local result=$({binary} process "$cmd" 2>/dev/null)
        if [[ -n "$result" ]]; then
            READLINE_LINE="$result"
        fi
    fi
    builtin accept-line
}

# Bind Enter key to our widget
bind -x '"\C-M": __text2cli_accept_line'
```

### 2. Zsh Hook

#### Architecture

```
User presses Enter
       ↓
Custom zle widget invoked
       ↓
Check BUFFER for trigger (@@@)
       ↓
┌─────────────────────────────────────┐
│ Trigger found?                      │
│   Yes → Transform command           │
│        → Set BUFFER to result       │
│        → zle .accept-line           │
│   No  → zle .accept-line            │
└─────────────────────────────────────┘
```

#### Implementation Details

1. **Widget Function**: Create a zle widget that:
   - Checks `$BUFFER` for trigger pattern
   - If found: transforms and updates `$BUFFER`
   - Calls the original accept-line widget

2. **Widget Registration**:
   - Use `zle -N __text2cli_accept_line` to define widget
   - Save original: `zle -A accept-line .text2cli.accept-line`
   - Replace: `zle -A __text2cli_accept_line accept-line`
   - Or bind directly: `bindkey '^M' __text2cli_accept_line`

3. **Buffer Manipulation**:
   - Use `BUFFER` variable for content
   - Use `CURSOR` for cursor position
   - After transformation, reset cursor to end

#### Generated Script Structure

```zsh
# text2cli zsh integration
# Add to ~/.zshrc

__text2cli_accept_line() {
    local cmd="$BUFFER"
    if [[ "$cmd" == @@@* ]]; then
        local result=$({binary} process "$cmd" 2>/dev/null)
        if [[ -n "$result" ]]; then
            BUFFER="$result"
            CURSOR=${#BUFFER}
        fi
    fi
    zle accept-line
}

zle -N __text2cli_accept_line
bindkey '^M' __text2cli_accept_line
```

### 3. PowerShell Hook

#### Architecture

```
User presses Enter
       ↓
PSReadLine key handler invoked
       ↓
Get buffer state
       ↓
Check for trigger (@@@)
       ↓
┌─────────────────────────────────────┐
│ Trigger found?                      │
│   Yes → Transform command           │
│        → Replace buffer             │
│        → AcceptLine                 │
│   No  → AcceptLine                  │
└─────────────────────────────────────┘
```

#### Implementation Details

1. **Key Handler**: Use `Set-PSReadLineKeyHandler` to intercept Enter:
   - Get buffer state with `GetBufferState()`
   - Check for trigger pattern
   - Transform and replace if found

2. **Buffer Operations**:
   - Use `[Microsoft.PowerShell.PSConsoleReadLine]::Replace()`
   - Use `AcceptLine()` to execute

3. **Error Handling**:
   - Wrap transformation in try/catch
   - Fall back to normal AcceptLine on error

#### Generated Script Structure

```powershell
# text2cli PowerShell integration
# Add to $PROFILE

function Invoke-Text2Cli {
    param([string]$Input)

    if ($Input -match '^@@@') {
        try {
            $result = & {binary} process $Input 2>$null
            if ($result) {
                return $result
            }
        } catch {
            # Fall through to return nothing
        }
    }
    return $null
}

Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {
    param($key, $arg)

    $line = $null
    $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

    if ($line -match '^@@@') {
        try {
            $result = & {binary} process $line 2>$null
            if ($result) {
                [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $result)
            }
        } catch {
            # Silently continue to accept line
        }
    }

    [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
}
```

## Files to Modify

- `src/shell/bash.rs` - Update `BashHook::generate()`
- `src/shell/zsh.rs` - Update `ZshHook::generate()`
- `src/shell/pwsh.rs` - Update `PwshHook::generate()`

## Testing Strategy

For each generated script:
1. Verify syntax correctness using shell-specific syntax checkers
   - Bash: `bash -n script.sh`
   - Zsh: `zsh -n script.zsh`
   - PowerShell: Use PowerShell parser
2. Manual testing in actual shell environments
3. Test edge cases:
   - Empty transformation result
   - Transformation errors
   - Non-trigger commands
   - Multi-line input (if applicable)

## Error Handling

All hooks should:
1. Silently fail on transformation errors
2. Fall back to normal execution if transformation fails
3. Not break existing shell functionality
