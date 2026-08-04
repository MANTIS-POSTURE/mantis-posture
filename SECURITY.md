# Security policy

## Scope

This policy covers the repository code, build scripts and MANTIS POSTURE release
artifacts. Public websites queried by the application and the user's Windows
system remain external dependencies.

## Supported versions

During the alpha, security fixes target the default branch and the latest
published release. An unsupported version may remain vulnerable.

## Reporting a vulnerability

Use GitHub Private Vulnerability Reporting when it is available in the
repository's **Security** tab. If it is unavailable, open only a “security
contact request” issue, without technical details or sensitive data, to
establish a private channel. Do not publish an exploitable proof of concept in
an issue.

When safe, include: version or commit, affected system, affected surface,
minimal reproduction steps, impact, required conditions and a reduced proof of
concept. Remove secrets, personal data, databases, exports, logs and raw
evidence before sending.

This personal project currently promises neither a response time nor a bug
bounty program. Receiving a report does not constitute recognition of a
vulnerability; triage is based on minimal reproduction and actual impact on the
alpha version.

## Stated guarantees and limitations

MANTIS does not collect passwords, tokens or cookies, automatically attribute a
public profile to a person, or send messages or GDPR requests. Collection is
bounded and results require human review. This does not protect a compromised
Windows machine and does not constitute a guarantee of anonymity or forensic
deletion.
