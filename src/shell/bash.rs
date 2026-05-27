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

__text2cli_preexec__() {{
    local cmd="$1"
    if [[ "$cmd" == @@@* ]]; then
        local result=$({} process "$cmd")
        if [[ -n "$result" ]]; then
            READLINE_LINE="$result"
        fi
    fi
}}

preexec_functions+=(__text2cli_preexec__)
"#,
            self.binary_name
        )
    }

    fn should_intercept(&self, input: &str, trigger: &str) -> bool {
        input.trim().starts_with(trigger)
    }
}
