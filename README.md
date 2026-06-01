<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/logo_website_dark.png">
  <img src="assets/logo_website_light.png" alt="AngaraBase" width="520">
</picture>

**Predictable by contract — HTAP database engine in Rust.**

![Rust](https://img.shields.io/badge/built_with-Rust-e8612a?logo=rust&logoColor=white)
![Linux](https://img.shields.io/badge/target-Linux_x86__64%20%7C%20aarch64-00b4d8)
![pgwire](https://img.shields.io/badge/protocol-PostgreSQL_pgwire-336791?logo=postgresql&logoColor=white)
![HTAP](https://img.shields.io/badge/workload-OLTP_%2B_HTAP-6366f1)
![License: BUSL-1.1](https://img.shields.io/badge/License-BUSL--1.1_%E2%86%92_Apache_2.0_by_2030-22c55e)
![RFC](https://img.shields.io/badge/design-via_RFCs-a78bfa)
![Status](https://img.shields.io/badge/status-dev_preview-f59e0b)
![Community Hub](https://img.shields.io/badge/this_repo-community_hub-34d399)

### 🌐 [angarabase.com](https://angarabase.com) · 📖 [angarabase.dev](https://angarabase.dev) · 🗺 [Roadmap](ROADMAP.md) · 📦 [Releases](../../releases) · 🐛 [Issues](../../issues) · 💬 [Discussions](../../discussions)

</div>

---

**This repository is the AngaraBase community hub.** Issues, discussions, installation packages and architectural contracts live here. Project website: [**angarabase.com**](https://angarabase.com). Canonical documentation: [**angarabase.dev**](https://angarabase.dev).

**AngaraBase is a Linux-native, PostgreSQL-wire-compatible (`pgwire`) relational HTAP engine, written in Rust
and built around a single principle: every behavior the database can exhibit is a contract — declared in code,
observable as a metric, enforced with an explicit `SQLSTATE`.** No silent degradation. No undocumented modes.
No paths that "usually work."

### What's in the box

- **PostgreSQL wire protocol (`pgwire`)** — works with stock `psql`, JDBC, `libpq`, `pgx`, `asyncpg`. No
  proprietary driver of our own.
- **UNDO-log MVCC** in the Oracle / InnoDB tradition — historical row versions live in a separate UNDO log,
  the heap holds only current versions. No `VACUUM`, no heap bloat, snapshot-deterministic visibility.
- **ARIES recovery** (Analysis → Redo → Undo, with CLR) — crash-consistent host migration and PITR through one
  recovery contour.
- **Fail-closed admission control** — eight named resource boundaries (memory, undo, write set, connections,
  …), each surfaced as a Prometheus metric and a unique `SQLSTATE`. The error arrives *before* the incident,
  not after.
- **Pluggable storage engines** behind one `TableEngine` trait — row store, AngaraMemory (three explicit
  durability tiers: `none` / `logged` / `snapshotted`), and AngaraColumn engine for HTAP workloads (columnar, SIMD-accelerated).
- **Linux-native observability** — Prometheus metrics on every contract boundary, USDT probes for `bpftrace` /
  `perf`, structured logs with stable field names.
- **Built-in security baseline** — `scram` authentication out of the box, RLS / audit / break-glass on the
  roadmap; no behavior is enabled silently.
- **Evidence-gated releases** — every release train closes on a 24-hour soak test and a pinned benchmark.
  Correctness is an artifact in `Releases`, not a marketing claim.

This makes AngaraBase suitable for workloads where **the cost of one unpredictable incident is higher than the
annual license of a commercial database** — fintech back-offices, billing, audit trails, ERP cores, regulated
systems of record. The trade-off is honest: a smaller, contract-bound SQL surface (see [What AngaraBase does
*not* do](#what-angarabase-does-not-do)) in exchange for behavior you can name, measure, and run a runbook
against.

---

## Where to find what

| You need | Where |
|---|---|
| **Project website** | [`angarabase.com`](https://angarabase.com) |
| **Documentation** (canonical) | [`angarabase.dev`](https://angarabase.dev) |
| **Installation packages** | [GitHub Releases](../../releases) · [`PACKAGES.md`](PACKAGES.md) |
| **Bugs, questions, feedback** | [GitHub Issues](../../issues) |
| **Discussions, use cases, ideas** | [GitHub Discussions](../../discussions) |
| **Architectural contracts** | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), [`contracts/`](contracts/) |
| **Project principles** | [`docs/PROJECT_PRINCIPLES.md`](docs/PROJECT_PRINCIPLES.md) |
| **Supported SQL subset** | [angarabase.dev → SQL Reference](https://angarabase.dev/sql-reference/) |
| **Current status & focus** | [`PROJECT_STATUS.md`](PROJECT_STATUS.md) |
| **Announcements (RU)** | [Telegram @angarabase](https://t.me/angarabase) |
| **Announcements (EN)** | X — *coming soon* |
| **Long-reads (RU)** | Habr — *coming soon* |

---

## What's in this repo, what's not

**✅ Here:**

- **Installation packages** via Releases — Linux `x86_64` / `aarch64`, `glibc ≥ 2.28`. Each package contains
  everything needed to run AngaraBase and to rebuild it under the terms of [`LICENSE`](LICENSE).
- **Architectural contracts** — `docs/ARCHITECTURE.md` and `contracts/*.rs` (Rust trait contracts for
  admission control and resource boundaries).
- **Supported SQL subset** and known-issues catalog with `SQLSTATE` codes.

**❌ Not here:**

- **Engine source code in git.** This is intentional: source is delivered inside the installation package
  under [`LICENSE`](LICENSE) terms, in order to keep one canonical distribution and avoid fragmenting forks
  during the early phase of the project.
- **Managed / cloud offering** — self-hosted Linux only, by design.
- **Internal planning corpus, RFC drafts, CI artifacts** — these live in the private development perimeter and
  ship as evidence in releases.

If you want to build AngaraBase from source, take the source package from [Releases](../../releases) — there
is no point cloning this repo for that, the code isn't here by design.

---

## Quickstart (~ 60 seconds)

```bash
# 1. Download and unpack (Linux x86_64 / aarch64, glibc >= 2.28)
#    https://github.com/angarabase/angarabase/releases
sudo mkdir -p /opt/angarabase
sudo tar -xzf angarabase-<version>-x86_64-unknown-linux-gnu.tar.gz -C /opt/angarabase

# 2. Create a dedicated system user and data directory
sudo useradd --system --no-create-home --shell /usr/sbin/nologin angarabase
sudo mkdir -p /var/lib/angarabase
sudo chown angarabase:angarabase /var/lib/angarabase

# 3. Initialize the instance (fail-closed: server will not start without --init)
sudo -u angarabase /opt/angarabase/bin/angarabase-server --init /var/lib/angarabase \
  --superuser angara_root --superuser-password 'change-me' \
  --auth-mode scram

# 4. Run (or wire into systemd — see angarabase.dev → Installation)
sudo -u angarabase /opt/angarabase/bin/angarabase-server \
  --config /var/lib/angarabase/config.toml

# 5. Connect with stock psql — pgwire-compatible
psql "host=127.0.0.1 port=5432 user=angara_root dbname=postgres"
```

Once connected, you should see something like this:

```
psql (15.6, server 0.6.5)
Type "help" for help.

postgres=# CREATE TABLE orders (
postgres(#   id      BIGSERIAL PRIMARY KEY,
postgres(#   user_id BIGINT NOT NULL,
postgres(#   amount  NUMERIC(12,2) NOT NULL,
postgres(#   state   TEXT NOT NULL DEFAULT 'pending',
postgres(#   created TIMESTAMPTZ NOT NULL DEFAULT now()
postgres(# );
CREATE TABLE

postgres=# INSERT INTO orders(user_id, amount) VALUES (1, 99.99), (2, 149.00);
INSERT 0 2

postgres=# EXPLAIN SELECT user_id, SUM(amount) FROM orders GROUP BY user_id;
                            QUERY PLAN
------------------------------------------------------------------
 HashAggregate  (cost=0.04 rows=2)
   Group Key: user_id
   ->  SeqScan on orders  (cost=0.00 rows=2)
(3 rows)

postgres=# SELECT * FROM angara_resource_usage();
      resource      | used | limit | pct
--------------------+------+-------+-----
 connections        |    1 |   100 |   1
 heap_mb            |    0 |  8192 |   0
 undo_mb            |    0 |  2048 |   0
 write_set_rows     |    0 | 50000 |   0
(4 rows)
```

> If a limit is breached, AngaraBase returns a deterministic `SQLSTATE` error *before* the incident —
> not after. See [`docs/PROJECT_PRINCIPLES.md`](docs/PROJECT_PRINCIPLES.md) §1 for the full contract.

Full installation path (RPM / DEB, systemd, native packages) — [angarabase.dev → Installation](https://angarabase.dev/operations/installation.html).

---

## What makes it predictable

1. **UNDO-log MVCC** — historical row versions live in a separate UNDO log; the heap holds only current
   versions. No `VACUUM`, no heap bloat. Visibility is snapshot-deterministic.
2. **Fail-closed by contract** — eight named resource boundaries (memory, undo, write set, connections, …),
   each with an explicit `SQLSTATE` and a Prometheus metric. The error arrives *before* the incident, not
   after. See [`docs/PROJECT_PRINCIPLES.md`](docs/PROJECT_PRINCIPLES.md) §1.
3. **ARIES recovery** — Analysis → Redo → Undo with CLR. Crash-consistent host migration and PITR through one
   recovery contour.
4. **Pluggable storage engines** — one `TableEngine` trait: row store, AngaraMemory with three durability
   tiers (`none` / `logged` / `snapshotted`), and AngaraColumn engine for analytics inside the same instance
   (columnar, SIMD-accelerated — HTAP without ETL).
5. **Evidence-gated releases** — every release train closes on a 24-hour soak test and a pinned benchmark. Not
   "probably faster" but a concrete `p99` on concrete hardware, archived in `Releases`.

Concept reference: [angarabase.dev → Concepts](https://angarabase.dev/concepts/).

---

## What AngaraBase does *not* do

A contract-bound SQL subset. What's supported is supported in full; what isn't returns an explicit `SQLSTATE
0A000` instead of a silent bypass:

- no `PL/pgSQL`, no triggers, no `LISTEN` / `NOTIFY`, no logical replication (on the current branch);
- no extensions (`pgvector`, `PostGIS`, …) — only built-in subsystems with declared contracts;
- no Windows / macOS production builds — Linux `x86_64` / `aarch64`, `glibc ≥ 2.28`;
- no managed / hosted offering — self-hosted only.

If "Postgres at 100%" today is a critical requirement, AngaraBase in its current state is not a fit. We say
this up front, not in a footnote. Full compatibility map:
[angarabase.dev → SQL Reference](https://angarabase.dev/sql-reference/).

---

## Community and contribution

- 🐛 **Found a bug or regression?** Open an [issue](../../issues) with reproduction steps. How to collect artifacts for a bug report: [angarabase.dev → Support](https://angarabase.dev/reference/support.html).
- 💬 **A use case, question or idea?** Come to [Discussions](../../discussions).
- 📖 **Want to help with documentation?** The canonical AngaraBook is edited in the private perimeter; submit
  corrections via an issue tagged `docs` — accepted edits ship in the next release.
- 🤝 **Design partner?** Reach out via an issue tagged `design-partner` — see the design-partner program on [angarabase.dev](https://angarabase.dev).

We do not accept code PRs into this repository — there is no source here by design. For architectural
proposals, post in [Discussions](../../discussions); accepted ideas go through the internal RFC process and
ship as part of a release train.

---

## Status & Roadmap

`v0.6.x` — **dev preview**. Suitable for supervised research pilots and design-partner engagements; not ready
for unsupervised production.

- Current branch and focus: [`PROJECT_STATUS.md`](PROJECT_STATUS.md)
- **Where we are headed → [`ROADMAP.md`](ROADMAP.md)**
- Releases: [GitHub Releases](../../releases)
- Known limitations and `SQLSTATE` codes: [angarabase.dev → Known Issues](https://angarabase.dev/reference/known-issues.html)

The high-level milestones: **v0.7** — Production-Ready + Open Beta (HA auto-failover, extended SQL, UDFs, CDC);
**v0.8** — single-node hardening + GA prep; **v0.9** — Public GA + Community Edition + transparent sharding.

---

## License

**Current release: [Business Source License 1.1](LICENSE) (BUSL-1.1)**

What this means today:

- ✅ **Use freely** — run, evaluate, build internal systems on top of AngaraBase.
- ✅ **Study the source** — delivered inside the installation package; read and learn from it.
- ❌ **No managed/hosted service** — may not offer AngaraBase as a commercial Database-as-a-Service.
- ❌ **No resale/OEM** — may not embed or resell without a separate agreement.

**Coming: tiered editions with free Community license key**

We are building a three-tier model:

| Edition | Price | Data limit | SLA | Source access |
|---|---|---|---|---|
| **Community** | Free | 100 GB / cluster | GitHub Issues | — |
| **Commercial** | Paid | By agreement | 8×5 support | — |
| **Enterprise** | Paid | Unlimited | 24×7 + dedicated engineer | Git repository |

- License key: auto-issued, annually renewable, **fully offline verification** (air-gap friendly)
- All core features available in Community: HA, sharding, HTAP (AngaraColumn), pgwire
- Government and educational institutions: extended limits on request

Until Community Edition launches, BUSL-1.1 governs all distributions. Questions about licensing or early access → [open an issue](../../issues) tagged `licensing`.

---

<sub>AngaraBase · Linux x86_64 / aarch64 · `glibc ≥ 2.28` · Predictable by contract · [angarabase.dev](https://angarabase.dev)</sub>
