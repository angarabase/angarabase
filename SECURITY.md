# Security Policy

## Supported versions

| Version | Security fixes |
|---------|---------------|
| `v0.6.x` (dev preview) | ✅ Actively patched |
| Earlier | ❌ End of life |

AngaraBase is currently in **dev preview**. Versions below `v0.6.0` are not supported.

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report security issues by emailing **security@angarabase.com** with:

- Subject: `[SECURITY] <one-line summary>`
- AngaraBase version (`angarabase-server --version`)
- Description of the vulnerability and potential impact
- Reproduction steps (minimal, without triggering a live exploit if possible)
- Any relevant SQLSTATE codes or log output

### What to expect

| Stage | Timeline |
|-------|---------|
| Acknowledgment | Within 3 business days |
| Triage and severity assessment | Within 7 business days |
| Fix target (confirmed vulnerability) | Aligned to next scheduled release or emergency patch |
| Public disclosure | Coordinated with reporter after fix ships |

We will credit reporters in the release notes unless you request anonymity.

## Scope

### In scope

- Memory safety issues in the server process (`angarabase-server`)
- Authentication bypass (SCRAM-SHA-256 implementation)
- Authorization escalation (RBAC / RLS bypass)
- WAL or crash-recovery correctness issues that can cause data corruption
- Denial-of-service via crafted SQL or pgwire messages (unbounded resource consumption)
- Contract violations: a resource boundary that does not fail-closed when documented to do so

### Out of scope

- Vulnerabilities in third-party dependencies not yet reported upstream
- Attacks that require physical access to the server
- Theoretical issues with no demonstrated exploit path
- Issues in versions below `v0.6.0`

## Security model

AngaraBase is a **self-hosted, Linux-native** database. The security model assumes:

- The OS user running `angarabase-server` is trusted.
- Network access to the pgwire port (`5432` by default) is controlled by the operator.
- `scram-sha-256` is the default and only supported authentication method;
  cleartext authentication is not implemented.

Managed / cloud deployment is not in scope for this version.

## Fail-closed guarantees

Every resource boundary in AngaraBase is designed to **fail closed** — it returns an
explicit `SQLSTATE` error rather than silently degrading. A security report that shows
a boundary failing open (accepting requests beyond its declared limit without error)
is in scope and treated with high severity.

See [`contracts/admission_control.rs`](contracts/admission_control.rs) for the full
table of resource boundaries and their SQLSTATE codes.
