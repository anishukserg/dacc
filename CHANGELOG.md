# Changelog

## 0.4.0 — unreleased

### Upgrade notes (breaking)

A project that uses dacc must do the following when it bumps the version:

- **`dacc_scan::journal::scan_journal` now takes two paths** — the single
  journal record and the former per-event directory — instead of one directory.
  In `build.rs`:

  ```rust
  dacc_scan::journal::scan_journal(
      &manifest.join("journal.toml"),
      &manifest.join("journal"),
      &work,
  )
  ```

- **`cargo dacc work start` no longer writes a commit.** The `started` event is
  written by `work land` together with `gate` and `landed`. The ceremony is now
  `commit` + `work land`; `work start` only validates. See `adr-2026-042`.

- **The journal is now one append-only record `doc/journal.toml`** — an array of
  `[[events]]` tables. The former per-event files in `doc/journal/` remain as
  frozen history and need **no migration**. See `adr-2026-043`.
