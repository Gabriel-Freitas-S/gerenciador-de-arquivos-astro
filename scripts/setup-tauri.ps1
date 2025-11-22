param(
    [string]$TauriVersion = "^2",
    [switch]$NoConfirm
)

$ErrorActionPreference = 'Stop'

function Assert-Command {
    param([string]$Name)
    $command = Get-Command $Name -ErrorAction SilentlyContinue
    return $null -ne $command
}

if (-not (Assert-Command 'cargo')) {
    throw 'cargo (Rust) não foi encontrado no PATH. Instale o Rust via https://rustup.rs/ e tente novamente.'
}

if (-not (Assert-Command 'cargo-binstall')) {
    Write-Warning 'cargo-binstall não foi encontrado.'
    Write-Host 'Instale com:' -ForegroundColor Yellow
    Write-Host '  irm https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-via-powershell.ps1 | iex' -ForegroundColor Cyan
    throw 'Instale cargo-binstall e execute novamente.'
}

$cargoArgs = @('binstall', "tauri-cli@$TauriVersion", '--secure')
if ($NoConfirm) {
    $cargoArgs += '--no-confirm'
}

Write-Host 'Instalando tauri-cli via cargo-binstall...' -ForegroundColor Cyan
& cargo @cargoArgs
