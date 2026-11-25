param(
    [Parameter(Mandatory = $true)][int] $AppPid,
    [switch] $Restored
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

function Find-Control([string] $Name, [System.Windows.Automation.ControlType] $Type) {
    $parts = New-Object 'System.Collections.Generic.List[System.Windows.Automation.Condition]'
    $parts.Add([System.Windows.Automation.PropertyCondition]::new(
        [System.Windows.Automation.AutomationElement]::NameProperty, $Name))
    if ($Type) {
        $parts.Add([System.Windows.Automation.PropertyCondition]::new(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty, $Type))
    }
    $condition = if ($parts.Count -eq 1) { $parts[0] } else {
        [System.Windows.Automation.AndCondition]::new($parts.ToArray())
    }
    for ($i = 0; $i -lt 50; $i++) {
        $process = Get-Process -Id $AppPid
        if ($process.MainWindowHandle -ne 0) {
            $window = [System.Windows.Automation.AutomationElement]::FromHandle($process.MainWindowHandle)
            $element = $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $condition)
            if ($element) { return $element }
        }
        Start-Sleep -Milliseconds 500
    }
    if ($window) {
        $elements = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants,
            [System.Windows.Automation.Condition]::TrueCondition)
        $elements | Where-Object { $_.Current.Name } | Select-Object -First 50 | ForEach-Object {
            Write-Host "UIA $($_.Current.ControlType.ProgrammaticName) $($_.Current.Name)"
        }
    }
    throw "O controle '$Name' não apareceu na janela instalada."
}

function Invoke-Control([string] $Name) {
    $button = Find-Control $Name ([System.Windows.Automation.ControlType]::Button)
    $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}

function Confirm-Progress {
    $path = Join-Path $env:LOCALAPPDATA 'io.github.honraan.rustlab\progresso.toml'
    for ($i = 0; $i -lt 30; $i++) {
        if (Test-Path $path) {
            $progress = Get-Content $path -Encoding UTF8 -Raw
            if ($progress -match '(?m)^["'']lesson:B-T01["'']\s*=\s*20\s*$') { return }
        }
        Start-Sleep -Milliseconds 300
    }
    throw 'O ledger TOML não registrou 20 XP para a primeira lição.'
}

if ($Restored) {
    $null = Find-Control 'Retomar trilha' ([System.Windows.Automation.ControlType]::Button)
    Confirm-Progress
    Write-Host 'XP e retomada confirmados após reabrir.'
    exit 0
}

Invoke-Control 'Começar a aprender'
$null = Find-Control 'O que é programar?'
Write-Host 'Lição canônica visível na WebView2 instalada.'
$option = Find-Control 'Compreender instruções, sequência e exemplos reais de software.' (
    [System.Windows.Automation.ControlType]::RadioButton)

Add-Type -AssemblyName System.Windows.Forms
$automationShell = New-Object -ComObject WScript.Shell
$null = $automationShell.AppActivate($AppPid)
$option.SetFocus()
[System.Windows.Forms.SendKeys]::SendWait(' ')
Start-Sleep -Milliseconds 300
$selection = $null
if ($option.TryGetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern, [ref]$selection) -and -not $selection.Current.IsSelected) {
    $selection.Select()
}
Invoke-Control 'Conferir respostas'
Confirm-Progress
$null = Find-Control 'Próxima lição' ([System.Windows.Automation.ControlType]::Button)
Write-Host 'Resposta, desbloqueio e XP confirmados no programa instalado.'
