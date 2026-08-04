param(
  [Parameter(Mandatory = $true)] [string] $BuildPython
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$work = Join-Path $PSScriptRoot '.build'
$venv = Join-Path $work 'venv'
$output = Join-Path $repo 'src-tauri\resources\user-scanner'

function Invoke-BuildCommand {
  param([string] $File, [string[]] $Arguments)
  & $File @Arguments
  if ($LASTEXITCODE -ne 0) { throw "La génération du bundle User Scanner a échoué (code $LASTEXITCODE)." }
}

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
Invoke-BuildCommand $python @('-m', 'PyInstaller', '--noconfirm', '--clean', '--onefile', '--name', 'user-scanner', '--collect-all', 'user_scanner', '--workpath', (Join-Path $work 'pyinstaller'), '--distpath', (Join-Path $work 'dist'), '--specpath', $work, (Join-Path $PSScriptRoot 'user_scanner_sidecar.py'))
New-Item -ItemType Directory -Force -Path $output | Out-Null
Copy-Item -LiteralPath (Join-Path $work 'dist\user-scanner.exe') -Destination (Join-Path $output 'user-scanner.exe') -Force
$license = Get-ChildItem -LiteralPath (Join-Path $venv 'Lib\site-packages') -Recurse -File -Filter 'LICENSE*' | Where-Object { $_.FullName -match 'user_scanner-.*dist-info' } | Select-Object -First 1
if ($null -eq $license) { throw 'La licence User Scanner est absente du paquet téléchargé.' }
Copy-Item -LiteralPath $license.FullName -Destination (Join-Path $output 'USER_SCANNER_LICENSE.txt') -Force
$hash = (& $python -c "import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], 'rb').read()).hexdigest())" (Join-Path $output 'user-scanner.exe')).Trim()
$manifest = @{ schema_version = 1; module_id = 'osint-email-platforms'; version = 'user-scanner-1.4.2.1'; file = 'user-scanner.exe'; sha256 = $hash; license_file = 'USER_SCANNER_LICENSE.txt' } | ConvertTo-Json
[System.IO.File]::WriteAllText((Join-Path $output 'manifest.json'), $manifest, [System.Text.UTF8Encoding]::new($false))
Write-Output 'Bundle User Scanner créé pour le paquet Tauri.'
