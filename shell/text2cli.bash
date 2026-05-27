# text2cli Bash Integration
# Add to ~/.bashrc: source /path/to/text2cli.bash

__text2cli_preexec__() {
    local cmd="$BASH_COMMAND"
    if [[ "$cmd" == @@@* ]]; then
        local result=$(text2cli process "$cmd" 2>/dev/null)
        if [[ -n "$result" && $? -eq 0 ]]; then
            # For bash, we need to use bind -x for buffer manipulation
            # This is a simplified version that echoes the command
            echo "$result"
        fi
    fi
}

# Use PROMPT_COMMAND for pre-execution hook
__text2cli_prompt_cmd__() {
    __text2cli_last_cmd=$BASH_COMMAND
}

# Alternative: use bind for Enter key
bind -x '"\C-m": "__text2cli_enter"'

__text2cli_enter() {
    local line="${READLINE_LINE:-}"
    if [[ "$line" == @@@* ]]; then
        local result=$(text2cli process "$line" 2>/dev/null)
        if [[ -n "$result" && $? -eq 0 ]]; then
            READLINE_LINE="$result"
            READLINE_POINT=${#READLINE_LINE}
        else
            builtin accept-line
        fi
    else
        builtin accept-line
    fi
}

echo "text2cli loaded. Use '@@@ <instruction>' to get command suggestions."
