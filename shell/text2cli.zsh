# text2cli Zsh Integration
# Add to ~/.zshrc: source /path/to/text2cli.zsh

__text2cli_preexec__() {
    local cmd="$1"
    if [[ "$cmd" == @@@* ]]; then
        local result=$(text2cli process "$cmd" 2>/dev/null)
        if [[ -n "$result" && $? -eq 0 ]]; then
            # Use print -z to inject into input buffer
            print -z "$result"
            # Return non-zero to cancel original command
            return 1
        fi
    fi
    return 0
}

# Add to preexec_functions
preexec_functions+=(__text2cli_preexec__)

echo "text2cli loaded. Use '@@@ <instruction>' to get command suggestions."
