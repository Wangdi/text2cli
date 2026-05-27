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

# Store the binary name
${{global:__Text2Cli_Binary__}} = "{}"

# Function to check if command should be intercepted
function Test-Text2CliTrigger {{
    param([string]$Line)
    return $Line -match '^@@@'
}}

# Function to process command through text2cli
function Invoke-Text2CliProcess {{
    param([string]$Input)

    try {{
        $result = & $__Text2Cli_Binary__ process $Input 2>$null
        if ($LASTEXITCODE -eq 0 -and $result) {{
            return $result
        }}
    }} catch {{
        Write-Host "[text2cli] Error: $_" -ForegroundColor Yellow
    }}
    return $null
}}

# PSReadLine handler for Enter key
function Set-Text2CliKeyHandler {{
    # Save the original Enter behavior
    $originalEnter = Get-PSReadLineKeyHandler -Chord Enter

    # Set up our custom handler
    Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {{
        param($key, $arg)

        # Get current line
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        # Check for trigger
        if (Test-Text2CliTrigger -Line $line) {{
            # Process through text2cli
            $result = Invoke-Text2CliProcess -Input $line

            if ($result) {{
                # Replace buffer with result
                [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $result)
                [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
                return
            }} else {{
                # Show error and clear
                [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, '')
                Write-Host "[text2cli] No command returned" -ForegroundColor Yellow
                return
            }}
        }}

        # Normal Enter behavior
        [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
    }}
}}

# Initialize the key handler
Set-Text2CliKeyHandler

# Utility function to manually suggest a command
function Invoke-Text2CliSuggest {{
    param([string]$Command)

    if (-not $Command) {{
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        $Command = $line
    }}

    if (Test-Text2CliTrigger -Line $Command) {{
        $result = Invoke-Text2CliProcess -Input $Command
        if ($result) {{
            # Insert into buffer
            [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $Command.Length, $result)
            Write-Host "[text2cli] Suggested: $result" -ForegroundColor Cyan
        }}
    }}
}}

# Cleanup function
function Remove-Text2CliHandler {{
    # Restore default Enter behavior
    Set-PSReadLineKeyHandler -Chord Enter -Function AcceptLine
}}
"#,
            self.binary_name
        )
    }

    fn should_intercept(&self, input: &str, trigger: &str) -> bool {
        input.trim().starts_with(trigger)
    }
}
