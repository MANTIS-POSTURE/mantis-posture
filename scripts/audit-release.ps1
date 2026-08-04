$ErrorActionPreference = 'Stop'

$required = @(
  'README.md',
  'LICENSE',
  'SECURITY.md',
  'CONTRIBUTING.md',
  'CODE_OF_CONDUCT.md',
  'THIRD-PARTY-NOTICES.md',
  '.gitignore',
  '.env.example'
)
foreach ($file in $required) {
  if (!(Test-Path -LiteralPath (Join-Path (Get-Location) $file))) {
    throw "Fichier open source requis absent: $file"
  }
}

$package = Get-Content -LiteralPath 'package.json' -Raw | ConvertFrom-Json
$lockVersion = (Select-String -LiteralPath 'package-lock.json' -Pattern '^\s*"version":\s*"([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value
$tauri = Get-Content -LiteralPath 'src-tauri\tauri.conf.json' -Raw | ConvertFrom-Json
$cargoVersion = (Select-String -LiteralPath 'src-tauri\Cargo.toml' -Pattern '^version\s*=\s*"([^"]+)"').Matches.Groups[1].Value
$versions = @($package.version, $lockVersion, $tauri.version, $cargoVersion) | Select-Object -Unique
if ($versions.Count -ne 1) { throw "Versions de release incohérentes: $($versions -join ', ')" }

$releaseFiles = @(git ls-files --cached --others --exclude-standard)
if ($LASTEXITCODE -ne 0) { throw 'Impossible de lire la liste des fichiers publiables.' }

$forbidden = $releaseFiles | Where-Object {
  $_ -match '(^|/)(\.env($|\.)|node_modules|target|dist|build|reports|scan-output|raw)(/|$)' -or
  $_ -match '\.(db|sqlite|sqlite3|log|msi|exe|dll|pdb|dmp|pem|key|p12|pfx)$' -or
  $_ -match '(^|/)(path/to/|Le script Python )'
}
if ($forbidden.Count -gt 0) {
  $forbidden | ForEach-Object { Write-Error "Artefact ou placeholder suivi par Git: $_" }
  throw 'Le dépôt contient des fichiers qui ne doivent pas être publiés.'
}

$textExtensions = @('.md', '.json', '.toml', '.ts', '.svelte', '.rs', '.ps1', '.js', '.mjs', '.yml', '.yaml', '.sql', '.html', '.css', '.txt')
$privateNamePattern = '(?i)\b(' + ('OU' + 'YED') + '|' + ('LA' + 'TAYA' + '\s+' + 'LAT' + 'CHIMY') + '|' + ('BO' + 'RY') + ')\b'
$secretPatterns = @(
  '-----BEGIN (RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----',
  '(?i)(api[_-]?key|secret|access[_-]?token|client[_-]?secret|private[_-]?key)\s*[:=]\s*["'']?[A-Za-z0-9_\-/+]{20,}',
  '(?i)(gh[pousr]_[A-Za-z0-9_\-]{20,}|xox[baprs]-[A-Za-z0-9-]{20,}|AKIA[0-9A-Z]{16})',
  '(?i)Authorization\s*:\s*Bearer\s+[A-Za-z0-9_\-.]{20,}',
  '(?i)C:\\Users\\[^\\\r\n]+\\(Documents|Desktop|Downloads|AppData)',
  $privateNamePattern
)
foreach ($file in $releaseFiles | Where-Object { $textExtensions -contains [IO.Path]::GetExtension($_).ToLowerInvariant() }) {
  if (!(Test-Path -LiteralPath $file)) { continue }
  $content = Get-Content -LiteralPath $file -Raw -ErrorAction Stop
  foreach ($pattern in $secretPatterns) {
    if ($content -match $pattern) { throw "Motif sensible détecté dans un fichier suivi: $file" }
  }
}

Write-Host "Release audit passed: $($releaseFiles.Count) publishable files checked."
