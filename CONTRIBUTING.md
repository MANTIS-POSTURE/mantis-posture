# Contributing to MANTIS POSTURE

MANTIS is a local-first project that handles sensitive public traces. A
contribution must therefore be useful, reproducible and explainable by a human.
Read [README.md](README.md) and [SECURITY.md](SECURITY.md) before modifying the
repository.

## Non-negotiable rules

- Never commit a password, token, cookie, SQLite database, personal data, real scan, raw evidence, log, export, binary or secret.
- Do not add a collector, AI or automation that creates a business object without consent and human review.
- Document the trust boundary and limitations of every new network request, Tauri command, migration or dependency.
- Use synthetic fixtures and fictional URLs in tests.

## Workflow

1. Create a short-lived branch from the default branch.
2. Describe the problem, security/privacy impact and affected surface before coding.
3. Add or update the relevant tests, migrations and documentation.
4. Keep changes focused and explain trade-offs in the pull request. Do not add a security badge or claim without a corresponding check.

## Local checks

```powershell
.\scripts\audit-release.ps1
npm ci
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Pull requests must state which commands were actually run, migration changes,
release impact and known limitations. Screenshots must use synthetic data only.


