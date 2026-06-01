# Contributing to AngaraBase

Thank you for your interest in AngaraBase.
Project website: [angarabase.com](https://angarabase.com) · Documentation: [angarabase.dev](https://angarabase.dev)

This repository is the **community hub** — issues, discussions, and installation packages.
Source code is not here by design; it ships inside installation packages from
[Releases](../../releases) under the terms of `LICENSE`.

---

## Ways to contribute

### 1. Report bugs and regressions

Open an [issue](../../issues/new?template=bug_report.md) using the bug report template.
Include the SQLSTATE code, server log window, and environment details.
How to collect artifacts: [angarabase.dev → Support](https://angarabase.dev/reference/support.html).

### 2. Ask questions and propose ideas

Use [Discussions](../../discussions). Not every question needs a formal proposal.
Browse existing discussions first — your topic may already have a thread.

### 3. Improve documentation

The canonical [AngaraBook](https://angarabase.dev) is edited in the private development
perimeter. To submit a correction:

1. Open an issue tagged `docs`.
2. Describe the inaccuracy precisely (section, current text, proposed correction, why).
3. Accepted corrections ship in the next release cycle.

Pull requests with documentation changes are not accepted here — the source-of-truth
lives elsewhere and accepting edits here would create divergence.

### 4. Architectural proposals

Post in [Discussions](../../discussions). Proposals that gain consensus go through the
internal RFC process. The RFC is then referenced in the release train where it lands.

RFCs are referenced in `docs/ARCHITECTURE.md` and documented on
[angarabase.dev](https://angarabase.dev) after graduation.

### 5. Design partnership

If you have a production OLTP workload and want early access in exchange for usage data
and feedback, open an [issue using the design-partner template](../../issues/new?template=design_partner.md).
Capacity: 5 active partners per release cycle.

---

## What we do *not* accept

- **Code PRs** — there is no source code in this repository.
- **Benchmark PRs** — benchmarks must be reproducible with pinned artifacts; contact us via Discussions.
- **Dependency bumps or security patches** — report vulnerabilities through `SECURITY.md`.

---

## Response time

- Bug reports: acknowledged within 5 business days.
- Discussions: no SLA, but actively monitored.
- Design partner inquiries: responded to within 10 business days.

---

## Code of conduct

We expect all participants to be respectful and constructive. Interactions that are
dismissive, aggressive, or off-topic will be closed without response.
The standard is senior-engineer discourse: direct, evidence-based, without drama.
