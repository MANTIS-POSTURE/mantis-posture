param(
  [Parameter(Mandatory = $true)] [string] $BuildPython
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$repo = Split-Path -Parent $root
$work = Join-Path $PSScriptRoot '.build'
$venv = Join-Path $work 'venv'
$output = Join-Path $repo 'src-tauri\resources\ddgs'

function Invoke-BuildCommand {
  param([string] $File, [string[]] $Arguments)
  & $File @Arguments
  if ($LASTEXITCODE -ne 0) { throw "La génération du bundle Empreinte Web a échoué (code $LASTEXITCODE)." }
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
Invoke-BuildCommand $python @('-m', 'PyInstaller', '--noconfirm', '--clean', '--onefile', '--name', 'ddgs-web', '--workpath', (Join-Path $work 'pyinstaller'), '--distpath', (Join-Path $work 'dist'), '--specpath', $work, (Join-Path $PSScriptRoot 'ddgs_web_sidecar.py'))
New-Item -ItemType Directory -Force -Path $output | Out-Null
Copy-Item -LiteralPath (Join-Path $work 'dist\ddgs-web.exe') -Destination (Join-Path $output 'ddgs-web.exe') -Force
$hash = (& $python -c "import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], 'rb').read()).hexdigest())" (Join-Path $output 'ddgs-web.exe')).Trim()
$manifest = @{ schema_version = 1; module_id = 'osint-web-footprint'; version = 'ddgs-9.14.4'; file = 'ddgs-web.exe'; sha256 = $hash } | ConvertTo-Json
[System.IO.File]::WriteAllText((Join-Path $output 'manifest.json'), $manifest, [System.Text.UTF8Encoding]::new($false))
Write-Output 'Bundle Empreinte Web créé pour le paquet Tauri.'
