$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$resources = @(
  @{ Directory = 'ddgs'; Module = 'osint-web-footprint'; File = 'ddgs-web.exe' },
  @{ Directory = 'user-scanner'; Module = 'osint-email-platforms'; File = 'user-scanner.exe' },
  @{ Directory = 'maigret'; Module = 'osint-username-profiles'; File = 'maigret-mantis.exe' }
)

foreach ($resource in $resources) {
  $directory = Join-Path $repo "src-tauri\resources\$($resource.Directory)"
  $manifestPath = Join-Path $directory 'manifest.json'
  if (!(Test-Path -LiteralPath $manifestPath)) { throw "Manifest absent: $manifestPath" }
  $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
  if ($manifest.schema_version -ne 1) { throw "Version de manifeste invalide: $manifestPath" }
  if ($manifest.module_id -ne $resource.Module) { throw "Module inattendu dans $manifestPath" }
  if ($manifest.file -ne $resource.File) { throw "Fichier inattendu dans $manifestPath" }
  if ($manifest.sha256 -notmatch '^[0-9a-fA-F]{64}$') { throw "SHA-256 invalide dans $manifestPath" }
  $binary = Join-Path $directory $manifest.file
  if (!(Test-Path -LiteralPath $binary)) { throw "Binaire absent: $binary" }
  $actual = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actual -ne $manifest.sha256.ToLowerInvariant()) { throw "SHA-256 incohérent: $binary" }
  if ($manifest.PSObject.Properties.Name -contains 'license_file') {
    $license = Join-Path $directory $manifest.license_file
    if (!(Test-Path -LiteralPath $license)) { throw "Licence absente: $license" }
  }
  Write-Host "Ressource valide: $($resource.Directory) ($actual)"
}
