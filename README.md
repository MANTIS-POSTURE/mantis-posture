# MANTIS POSTURE

> Read your traces. Stay in control.

MANTIS POSTURE is a local, experimental Windows desktop application that helps
you examine public traces associated with your own identifiers. It separates
declared data, collected observations and human decisions. A result remains
potential until it has been reviewed.

The project is a personal alpha born from curiosity and a desire to understand.
I am neither a professional developer nor an OSINT expert. The repository is
public so that the method can be checked, feedback can be received and a useful
tool can be shared without turning privacy into a product.

MANTIS is not a pentesting tool, an identity-attribution service or a *Minority
Report*-style oracle. It does not promise anonymity, absolute protection or
complete coverage of the Web.

## What works in the alpha

- dossiers and identities composed only of values entered by the user;
- authorized scans of email addresses, usernames, names and other supported values;
- searches for known breaches, potential accounts, public profiles and Web mentions using XposedOrNot, User Scanner, DDGS, Maigret and documented direct public sources;
- local retention of sources, dates, evidence, uncertainty and contradictions;
- human review before creating any exposure, incident or action;
- local tracking of actions, GDPR request drafts and reports;
- a relationship graph based on retained items;
- optional local analysis with constrained output and deterministic fallback.

MANTIS does not send emails, forms or GDPR requests. It does not collect
passwords, cookies or tokens. No scan, model or collector creates a business
conclusion on its own.

## Important limitations

- Windows is the supported target. Other platforms have not been validated.
- A network search reveals the machine's IP address and the searched value to the service being queried. *Local-first* therefore does not mean *offline*.
- Public sources change, fail and produce false positives. An exact profile may belong to a namesake.
- MANTIS does not encrypt SQLite, evidence or exports. Their protection depends on the Windows account, permissions and disk encryption.
- Local AI can be wrong. It explains and sorts; it does not create facts, evidence or decisions.
- No independent security audit, certification or legal compliance is claimed.

See [SECURITY.md](SECURITY.md) before interpreting a result.

## Installing the alpha

When a GitHub release is published:

1. download the Windows installer and SHA-256 checksum file from the **Releases** page;
2. verify the installer's SHA-256 hash;
3. run the installer.

```powershell
Get-FileHash .\mantis-posture_0.1.0_x64-setup.exe -Algorithm SHA256
```

The alpha may be distributed without an Authenticode signature. Windows
SmartScreen may then display a warning: verify that the file comes from the
expected GitHub release and that its hash matches before continuing.

Data for the installed version is stored under
`%APPDATA%\com.mantisposture.desktop`. Uninstalling the application must not be
considered a forensic deletion of this data.

## Developing and running from source

Requirements: Windows, Node.js 22, npm, stable Rust and the system dependencies
for [Tauri 2](https://v2.tauri.app/start/prerequisites/).

```powershell
npm ci
npm run tauri dev
```

Development mode stores its data in `src-tauri/.mantis-dev-data`. `npm run dev`
only starts the Web interface; Tauri and SQLite commands require
`npm run tauri dev`.

Local checks:

```powershell
.\scripts\audit-release.ps1
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml --locked
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

Building the Windows installer:

```powershell
.\scripts\build-sidecars.ps1 -BuildPython python
.\scripts\verify-release-resources.ps1
npm run tauri build -- --bundles nsis
```

Python 3.12 is required to build the sidecars, not to use the installed
application. Generated executables are not versioned.

## Privacy and security

The SQLite database, raw evidence, OSINT components and local models remain in
the application's private directory. Network requests are bounded per
collector; local and private destinations are refused during Web checks. These
controls reduce risk, but they do not make a compromised machine safe.

Vulnerabilities must be reported according to [SECURITY.md](SECURITY.md),
without personal data or raw evidence. Distributed dependencies and resources
are described in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

## Contributing

Focused contributions are welcome, especially around tests, network and file
boundary safety, accessibility and evidence clarity. Read
[CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

## License

MANTIS POSTURE code is released under the [MIT](LICENSE) license. Dependencies,
sidecars, trademarks and third-party assets retain their own licenses and rights.
