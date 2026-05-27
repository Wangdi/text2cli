use super::ShellHook;

pub struct BashHook {
    binary_name: String,
}

impl BashHook {
    pub fn new(binary_name: impl Into<String>) -> Self {
        Self {
            binary_name: binary_name.into(),
        }
    }
}

impl ShellHook for BashHook {
    fn name(&self) -> &str {
        "bash"
    }

    fn generate(&self) -> String {
        format!(
            r#"
# text2cli bash integration
# Add to ~/.bashrc

# Store the binary name
__TEXT2CLI_BINARY__="{}"

# Function to intercept and process commands
__text2cli_accept_line__() {{
    local line="${{READLINE_LINE:-}}"

    # Check if line starts with trigger
    if [[ "$line" == @@@* ]]; then
        # Process the command through text2cli
        local result
        result=$($__TEXT2CLI_BINARY__ process "$line" 2>/dev/null)

        if [[ -n "$result" ]]; then
            # Replace the line with the result
            READLINE_LINE="$result"
            READLINE_POINT=${{#result}}
        else
            # If no result, clear the line
            READLINE_LINE=""
            READLINE_POINT=0
        fi
    else
        # Normal behavior - execute the command
        builtin history -s "$line"
        eval "$line"
    fi
}}

# Bind the function to Enter key
bind -x '"\r": __text2cli_accept_line__'

# Alternative: use DEBUG trap for pre-execution
__text2cli_debug_trap__() {{
    local cmd="${{BASH_COMMAND:-}}"

    # Only intercept if command starts with trigger
    if [[ "$cmd" == @@@* ]]; then
        # Get the result
        local result
        result=$($__TEXT2CLI_BINARY__ process "$cmd" 2>/dev/null)

        if [[ -n "$result" ]]; then
            # Print the suggested command
            echo ""
            echo "Suggested: $result"
            echo "Press Enter to execute, or type a new command"
        fi
    fi
}}

# Uncomment to use DEBUG trap instead:
# trap '__text2cli_debug_trap__' DEBUG
"#,
            self.binary_name
        )
    }

    fn should_intercept(&self, input: &str, trigger: &str) -> bool {
        input.trim().starts_with(trigger)
    }
}
