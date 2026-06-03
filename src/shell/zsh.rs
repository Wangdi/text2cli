use super::ShellHook;

pub struct ZshHook {
    binary_name: String,
}

impl ZshHook {
    pub fn new(binary_name: impl Into<String>) -> Self {
        Self {
            binary_name: binary_name.into(),
        }
    }
}

impl ShellHook for ZshHook {
    fn name(&self) -> &str {
        "zsh"
    }

    fn generate(&self) -> String {
        format!(
            r#"
# text2cli zsh integration
# Add to ~/.zshrc

# Store the binary name
typeset -g __TEXT2CLI_BINARY__="{}"

# Widget function to handle Enter key
__text2cli_accept_line__() {{
    local line="${{BUFFER:-}}"

    # Check if line starts with trigger
    if [[ "$line" == @@@* ]]; then
        # Process the command through text2cli
        local result
        result=$($__TEXT2CLI_BINARY__ process "$line" 2>/dev/null)

        if [[ -n "$result" ]]; then
            # Replace buffer with the result
            BUFFER="$result"
            CURSOR=${{#BUFFER}}

            # Show what happened
            zle -M "Command ready: $result"
        else
            # Clear buffer if no result
            BUFFER=""
            CURSOR=0
            zle -M "text2cli: No command returned"
        fi
    else
        # Normal behavior
        zle accept-line
    fi
}}

# Create and bind the widget
zle -N __text2cli_accept_line__
bindkey '^M' __text2cli_accept_line__

# Alternative: preexec hook for non-interactive interception
__text2cli_preexec__() {{
    local cmd="$1"

    if [[ "$cmd" == @@@* ]]; then
        # Get suggested command
        local result
        result=$($__TEXT2CLI_BINARY__ process "$cmd" 2>/dev/null)

        if [[ -n "$result" ]]; then
            # Cancel current command and inject new one
            print -z "$result"
            zle -M "Suggested: $result (press Enter)"
        fi
    fi
}}

# Uncomment to use preexec instead:
# preexec_functions+=(__text2cli_preexec__)

# Utility function to manually trigger
text2cli-suggest() {{
    local cmd="$1"
    if [[ -z "$cmd" ]]; then
        cmd="$BUFFER"
    fi

    local result
    result=$($__TEXT2CLI_BINARY__ process "$cmd" 2>/dev/null)

    if [[ -n "$result" ]]; then
        BUFFER="$result"
        CURSOR=${{#BUFFER}}
    fi
}}
"#,
            self.binary_name
        )
    }

    fn should_intercept(&self, input: &str, trigger: &str) -> bool {
        input.trim().starts_with(trigger)
    }
}
