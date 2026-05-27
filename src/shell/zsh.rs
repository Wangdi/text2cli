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

__text2cli_preexec__() {{
    local cmd="$1"
    if [[ "$cmd" == @@@* ]]; then
        local result=$({} process "$cmd")
        if [[ -n "$result" ]]; then
            print -z "$result"
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
