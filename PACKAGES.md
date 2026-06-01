# Installation packages

AngaraBase is distributed as ready-to-run Linux packages.
All packages include source code under the terms of [`LICENSE`](LICENSE),
so you can rebuild from the same tarball.
Project website: [angarabase.com](https://angarabase.com) · Documentation: [angarabase.dev](https://angarabase.dev)

**Supported platforms:** Linux `x86_64` / `aarch64`, `glibc ≥ 2.28`.  
**Supported distributions:** RHEL / Rocky / Alma 9+, Fedora 39+, Debian 12+, Ubuntu 22.04+,
Gentoo (rolling).

---

## Tarball (universal)

The tarball works on any compatible Linux system without a package manager.

```bash
# Download from GitHub Releases (replace <version> and <arch>):
curl -LO https://github.com/angarabase/angarabase/releases/download/v<version>/angarabase-<version>-<arch>-unknown-linux-gnu.tar.gz
curl -LO https://github.com/angarabase/angarabase/releases/download/v<version>/SHA256SUMS

# Verify integrity:
sha256sum -c SHA256SUMS --ignore-missing

# Extract and run:
mkdir -p /opt/angarabase
tar -xzf angarabase-<version>-<arch>-unknown-linux-gnu.tar.gz -C /opt/angarabase
/opt/angarabase/bin/angarabase-server --version
```

All packages are signed with the AngaraBase release key.
GPG verification instructions: [angarabase.dev → Installation](https://angarabase.dev/operations/installation.html#gpg-verification).

---

## RPM (RHEL / Rocky / Alma / Fedora)

### Configure the repository

```bash
# RHEL 9 / Rocky 9 / Alma 9:
sudo curl -o /etc/yum.repos.d/angarabase.repo \
  https://packages.angarabase.dev/rpm/el9/angarabase.repo

# Fedora 39+:
sudo curl -o /etc/yum.repos.d/angarabase.repo \
  https://packages.angarabase.dev/rpm/fedora39/angarabase.repo
```

The repo file configures the AngaraBase package repository and imports the signing key.

### Install

```bash
sudo dnf install angarabase
```

### Update

```bash
sudo dnf upgrade angarabase
```

### Repository file contents

```ini
[angarabase]
name=AngaraBase
baseurl=https://packages.angarabase.dev/rpm/el9/$basearch/
enabled=1
gpgcheck=1
gpgkey=https://packages.angarabase.dev/rpm/RPM-GPG-KEY-angarabase
repo_gpgcheck=0
metadata_expire=6h
```

---

## DEB (Debian / Ubuntu)

```bash
# Import the signing key:
curl -fsSL https://packages.angarabase.dev/deb/GPG-KEY-angarabase \
  | sudo gpg --dearmor -o /usr/share/keyrings/angarabase.gpg

# Add the repository:
echo "deb [signed-by=/usr/share/keyrings/angarabase.gpg] \
  https://packages.angarabase.dev/deb stable main" \
  | sudo tee /etc/apt/sources.list.d/angarabase.list

# Install:
sudo apt-get update && sudo apt-get install angarabase
```

---

## Gentoo overlay

AngaraBase is available as a Gentoo Portage overlay:
**[github.com/angarabase/angarabase-overlay](https://github.com/angarabase/angarabase-overlay)**

### Add the overlay (eselect-repository)

```bash
sudo eselect repository add angarabase git \
  https://github.com/angarabase/angarabase-overlay.git
sudo emaint -r angarabase sync
```

### Install

```bash
# Sync the overlay:
sudo emerge --sync angarabase

# Install (dev-db/angarabase):
sudo emerge dev-db/angarabase
```

### USE flags

| Flag | Default | Description |
|------|---------|-------------|
| `usdt` | `off` | Enable USDT probes for `bpftrace` / `perf` (requires kernel ≥ 5.8) |
| `scram` | `on` | SCRAM-SHA-256 authentication (disable only for testing) |
| `systemd` | `off` | Install systemd unit file |
| `openrc` | `off` | Install OpenRC init script |

---

## systemd (post-install)

```bash
# Initialize a new instance (fail-closed: the server will not start without --init):
angarabase-server --init /var/lib/angarabase \
  --superuser angara_root --superuser-password 'change-me' \
  --auth-mode scram

# Enable and start:
sudo systemctl enable --now angarabase

# Connect:
psql "host=127.0.0.1 port=5432 user=angara_root dbname=postgres"
```

Full installation guide: [angarabase.dev → Installation](https://angarabase.dev/operations/installation.html).

---

## Package contents

Every package (tarball, RPM, DEB, ebuild) contains:

| Path | Contents |
|------|---------|
| `bin/angarabase-server` | The database server binary |
| `bin/angara` | The CLI client |
| `share/doc/angarabase/` | Offline copy of AngaraBook |
| `share/angarabase/contracts/` | Public contract specification files |
| `src/` | Full source code (see `LICENSE`) |
| `LICENSE` | License text |
| `SHA256SUMS` | Checksums of all included files |

---

## Verify a package (GPG)

```bash
# Import the AngaraBase release key (first time only):
gpg --recv-keys <FINGERPRINT>
# or:
curl -fsSL https://packages.angarabase.dev/GPG-KEY-angarabase | gpg --import

# Verify the tarball signature:
gpg --verify angarabase-<version>-x86_64-unknown-linux-gnu.tar.gz.asc
```

The full key fingerprint and release signing policy:
[angarabase.dev → Security](https://angarabase.dev/security/overview.html).
