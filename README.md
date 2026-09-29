# DACC

**Docs as Compiled Code** — методология и крейты Rust, где решения, спецификации и план работ — это код: ссылка на документ — путь к константе, а висячая ссылка не компилируется.

*A methodology and Rust crates where decisions, specifications and the work plan are code — a document reference is a path to a constant, and a dangling reference does not compile.*

[![crates.io](https://img.shields.io/crates/v/dacc?label=crates.io&color=blue)](https://crates.io/crates/dacc)
[![gate](https://github.com/anishukserg/dacc/actions/workflows/gate.yml/badge.svg)](https://github.com/anishukserg/dacc/actions/workflows/gate.yml)
[![mutants](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fanishukserg%2Fdacc%2Fmutation-badge%2Fmutation.json&label=mutants)](https://github.com/anishukserg/dacc/actions/workflows/mutation.yml)

## С чего читать

- [GUARANTEES.md](GUARANTEES.md) — что гарантируется и чем; читать первым.
- [DACC.md](DACC.md) — методология: слои, инварианты, уровни внедрения.
- [doc](doc) — реестр самого DACC: решения, спецификации, план и журнал, записанные по его же правилам.

## Крейты

[dacc](crates/dacc) — фасад · [dacc-core](crates/dacc-core) · [dacc-knowledge](crates/dacc-knowledge) · [dacc-work](crates/dacc-work) · [dacc-scan](crates/dacc-scan) · [dacc-derive](crates/dacc-derive) · [dacc-journal](crates/dacc-journal) · [dacc-access](crates/dacc-access) · [dacc-cli](crates/dacc-cli)

## Быстрый старт

```bash
cargo test --workspace              # тесты и doctest
cargo run -p dacc-cli -- gate       # калитка коммита
```

Подробнее о сборке, калитке и коммитах — [CONTRIBUTING.md](CONTRIBUTING.md).

## Участие

Как предложить изменение — [CONTRIBUTING.md](CONTRIBUTING.md); об уязвимостях — приватно, [SECURITY.md](SECURITY.md). Лицензия — [Apache-2.0](LICENSE).

