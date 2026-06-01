# Releases

All AngaraBase releases are published as **GitHub Releases** in this repository.
Every release includes the full source code under [`LICENSE`](LICENSE),
a SHA-256 manifest, and a GPG signature.

---

## Release naming

```
v<major>.<minor>.<patch>[-<label>]

Examples:
  v0.6.5    — release train ship
  v0.6.5-rc1 — release candidate
```

The `v0.x` series is **dev preview**: API contracts are evolving,
but SQLSTATE codes and resource boundary semantics are stable within a minor.

---

## What's in each release

| Artifact | Contents |
|----------|---------|
| `angarabase-<ver>-x86_64-unknown-linux-gnu.tar.gz` | Server binary, CLI, offline docs, source, contracts |
| `angarabase-<ver>-aarch64-unknown-linux-gnu.tar.gz` | Same, ARM64 |
| `angarabase-<ver>-x86_64.rpm` | RPM for RHEL / Rocky / Alma / Fedora |
| `angarabase-<ver>-aarch64.rpm` | RPM, ARM64 |
| `angarabase-<ver>-amd64.deb` | DEB for Debian / Ubuntu |
| `angarabase-<ver>-arm64.deb` | DEB, ARM64 |
| `SHA256SUMS` | Checksums for all artifacts |
| `SHA256SUMS.asc` | GPG signature over `SHA256SUMS` |
| `EVIDENCE.tar.gz` | Soak-test results + pinned benchmark (closes each release train) |

Package manager repos (RPM / DEB / Gentoo) are updated automatically
within 30 minutes of a GitHub Release publish.

---

## Verify a release

```bash
# Import the AngaraBase release key (first time only):
curl -fsSL https://packages.angarabase.dev/GPG-KEY-angarabase | gpg --import

# Download artifacts:
curl -LO https://github.com/angarabase/angarabase/releases/download/v<version>/SHA256SUMS
curl -LO https://github.com/angarabase/angarabase/releases/download/v<version>/SHA256SUMS.asc
curl -LO https://github.com/angarabase/angarabase/releases/download/v<version>/angarabase-<version>-x86_64-unknown-linux-gnu.tar.gz

# Verify GPG signature:
gpg --verify SHA256SUMS.asc SHA256SUMS

# Verify file integrity:
sha256sum -c SHA256SUMS --ignore-missing
```

The release key fingerprint and signing policy:
[angarabase.dev → Security](https://angarabase.dev/reference/security.html).

---

## Package manager repositories

For ongoing installations managed by a package manager, see [`PACKAGES.md`](PACKAGES.md).

| Channel | URL |
|---------|-----|
| RPM (RHEL/Rocky/Alma 9) | `https://packages.angarabase.dev/rpm/el9/` |
| RPM (Fedora 39+) | `https://packages.angarabase.dev/rpm/fedora39/` |
| DEB (Debian/Ubuntu) | `https://packages.angarabase.dev/deb/` |
| Gentoo overlay | `https://github.com/angarabase/angarabase-overlay` |

---

## Release notes format

Each GitHub Release body follows this structure:

```
## AngaraBase v<version> — <date>

### What's in this release
<1–3 sentences: what changed from the user's perspective>

### Evidence
- Soak test: <duration>, <load profile>, <p99 result> — archived in EVIDENCE.tar.gz
- Benchmark ref: <tool/config> on <hardware> — archived in EVIDENCE.tar.gz

### Installation
See PACKAGES.md or the tarball.

### Known limitations
<explicit list of known issues with SQLSTATE codes where applicable>

### Supported SQL subset
docs/technical_specs/v1_supported_sql_subset.md (also at angarabase.dev/reference/sql-subset.html)

### Upgrade notes
<breaking changes, if any, with SQLSTATE codes for new error conditions>
```

---

## Source code

Source code is not stored in git by design — it ships inside every installation package.
If you want to build AngaraBase from source, download the tarball from Releases
and follow the build instructions in `src/BUILDING.md` inside the archive.

This design prevents fork fragmentation during the early phase of the project.
