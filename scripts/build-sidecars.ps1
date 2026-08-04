param(
  [string] $BuildPython = 'python'
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$sidecars = @(
  @{ Name = 'DDGS'; Path = 'tools\ddgs-sidecar\build-bundle.ps1' },
  @{ Name = 'User Scanner'; Path = 'tools\user-scanner-sidecar\build-bundle.ps1' },
  @{ Name = 'Maigret'; Path = 'tools\maigret-sidecar\build-bundle.ps1' }
)

foreach ($sidecar in $sidecars) {
  $script = Join-Path $repo $sidecar.Path
  if (!(Test-Path -LiteralPath $script)) {
    throw "Le script de build $($sidecar.Name) est introuvable: $script"
  }
  Write-Host "Construction du sidecar $($sidecar.Name)..."
  & $script -BuildPython $BuildPython
  if ($LASTEXITCODE -ne 0) { throw "La construction du sidecar $($sidecar.Name) a échoué." }
}

& (Join-Path $PSScriptRoot 'verify-release-resources.ps1')
