param(
  [Parameter(Mandatory = $true)] [string] $BuildPython,
  [switch] $FinalizeOnly
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$work = Join-Path $PSScriptRoot '.build'
$venv = Join-Path $work 'venv'
$output = Join-Path $repo 'src-tauri\resources\maigret'

function Invoke-BuildCommand {
  param([string] $File, [string[]] $Arguments)
  & $File @Arguments
  if ($LASTEXITCODE -ne 0) { throw "La génération du bundle Maigret a échoué (code $LASTEXITCODE)." }
}

if (!$FinalizeOnly) {
  $resolvedBuildPython = if (Test-Path -LiteralPath $BuildPython) {
    (Resolve-Path -LiteralPath $BuildPython).Path
  } else {
    (Get-Command $BuildPython -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty Source)
  }
  if ([string]::IsNullOrWhiteSpace($resolvedBuildPython)) { throw "Le runtime de build privé indiqué est introuvable : $BuildPython" }
  $BuildPython = $resolvedBuildPython
  Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $work | Out-Null
  Invoke-BuildCommand $BuildPython @('-m', 'venv', $venv)
  $python = Join-Path $venv 'Scripts\python.exe'
  Invoke-BuildCommand $python @('-m', 'pip', 'install', '--disable-pip-version-check', '--requirement', (Join-Path $PSScriptRoot 'requirements.lock'))
  Invoke-BuildCommand $python @('-m', 'PyInstaller', '--noconfirm', '--clean', '--onefile', '--name', 'maigret-mantis', '--collect-all', 'maigret', '--collect-all', 'socid_extractor', '--workpath', (Join-Path $work 'pyinstaller'), '--distpath', (Join-Path $work 'dist'), '--specpath', $work, (Join-Path $PSScriptRoot 'maigret_sidecar.py'))
}
$python = Join-Path $venv 'Scripts\python.exe'
if (!(Test-Path -LiteralPath (Join-Path $work 'dist\maigret-mantis.exe')) -or !(Test-Path -LiteralPath $python)) { throw 'Le build Maigret à finaliser est incomplet.' }
New-Item -ItemType Directory -Force -Path $output | Out-Null
Copy-Item -LiteralPath (Join-Path $work 'dist\maigret-mantis.exe') -Destination (Join-Path $output 'maigret-mantis.exe') -Force
$license = Get-ChildItem -LiteralPath (Join-Path $venv 'Lib\site-packages') -Recurse -File -Filter 'LICENSE*' | Where-Object { $_.FullName -match 'maigret-.*dist-info' } | Select-Object -First 1
if ($null -eq $license) { throw 'La licence Maigret est absente du paquet téléchargé.' }
Copy-Item -LiteralPath $license.FullName -Destination (Join-Path $output 'MAIGRET_LICENSE.txt') -Force
$hash = (& $python -c "import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], 'rb').read()).hexdigest())" (Join-Path $output 'maigret-mantis.exe')).Trim()
$manifest = @{ schema_version = 1; module_id = 'osint-username-profiles'; version = 'maigret-0.6.3'; file = 'maigret-mantis.exe'; sha256 = $hash; license_file = 'MAIGRET_LICENSE.txt' } | ConvertTo-Json
[System.IO.File]::WriteAllText((Join-Path $output 'manifest.json'), $manifest, [System.Text.UTF8Encoding]::new($false))
Write-Output 'Bundle Maigret créé pour le paquet Tauri.'
