---
name: Bug report / regression
about: Something that should work doesn't, or behavior changed unexpectedly
title: "[BUG] "
labels: bug
assignees: ''
---

## Summary

<!-- One or two sentences. What failed? What was expected? -->

## Version

```
angarabase-server --version
```

<!-- Paste the full output, including build metadata if any. -->

## Environment

- **OS / distro:** (e.g. Rocky Linux 9.3, Fedora 40, Debian 12)
- **Architecture:** x86_64 / aarch64
- **glibc version:** (`ldd --version`)
- **Installed via:** tarball / RPM / Gentoo overlay / other

## Reproduction steps

```sql
-- minimal reproduction case:

```

## Actual behavior

<!-- What actually happened. Include the full SQLSTATE + error message if applicable. -->

## Expected behavior

<!-- What should have happened, per the documentation or prior behavior. -->

## Artifacts (attach if possible)

Collect using the support runbook at [angarabase.dev → Support](https://angarabase.dev/reference/support.html):

- [ ] Server log (structured JSON, trimmed to the relevant window)
- [ ] `EXPLAIN` output (for query-behavior issues)
- [ ] Prometheus metric snapshot at the time of the failure
- [ ] `angarabase-server` config (with credentials redacted)
- [ ] Any USDT / `bpftrace` trace if the issue involves latency

## Additional context

<!-- RFC references, related issues, related config knobs, etc. -->
