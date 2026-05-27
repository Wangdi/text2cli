# text2cli PowerShell Integration
# Source this file or add to $PROFILE

function Invoke-Text2Cli {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Input
    )

    $trigger = "@@@"
    if ($Input -match "^$trigger") {
        $content = $Input.Substring($trigger.Length).Trim()
        $result = text2cli process $content
        if ($result) {
            return $result
        }
    }
    return $Input
}

# Key handler for Enter key with trigger detection
Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {
    param($key, $arg)

    $line = $null
    $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

    if ($line -match '^@@@') {
        $result = text2cli process $line
        if ($result) {
            [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $result)
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
            return
        }
    }

    [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
}

Write-Host "text2cli loaded. Use '@@@ <instruction>' to get command suggestions."
