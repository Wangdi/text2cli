use super::ShellHook;

pub struct PwshHook {
    binary_name: String,
}

impl PwshHook {
    pub fn new(binary_name: impl Into<String>) -> Self {
        Self {
            binary_name: binary_name.into(),
        }
    }
}

impl ShellHook for PwshHook {
    fn name(&self) -> &str {
        "pwsh"
    }

    fn generate(&self) -> String {
        format!(
            r#"
# text2cli PowerShell integration
# Add to $PROFILE

function Invoke-Text2Cli {{
    param([string]$Input)

    if ($Input -match '^@@@') {{
        $result = {} process $Input
        if ($result) {{
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert($result)
        }}
    }}
}}

# Register as a command validation handler
Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {{
    param($key, $arg)

    $line = $null
    $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

    if ($line -match '^@@@') {{
        $result = {} process $line
        if ($result) {{
            [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $result)
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
            return
        }}
    }}

    [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
}}
"#,
            self.binary_name, self.binary_name
        )
    }

    fn should_intercept(&self, input: &str, trigger: &str) -> bool {
        input.trim().starts_with(trigger)
    }
}
