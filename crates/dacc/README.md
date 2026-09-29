# dacc

Фасадный крейт [DACC](https://github.com/anishukserg/dacc) — единая точка входа: публичная поверхность слоёв знания и работы одним `cargo add dacc`.

- **Одна зависимость** вместо набора `dacc-core`, `dacc-knowledge`, `dacc-work`, `dacc-scan`, `dacc-access`, `dacc-journal` — переэкспорт имён в одном пространстве.
- **Макросы** (`adr!`, `rfc!`, `work!`, `slice!`, `thrust!`, `declare_taxonomy!`, …) тоже переэкспортированы; их `$crate` — крейт-источник, поэтому при разметке реестра конкретные крейты остаются зависимостями.

```rust
use dacc::{adr, nonempty_str, taxon};
```

Разделение на крейты — по уровням: слой знания ([dacc-knowledge](https://github.com/anishukserg/dacc/tree/master/crates/dacc-knowledge)), слой работы ([dacc-work](https://github.com/anishukserg/dacc/tree/master/crates/dacc-work)), журнал ([dacc-journal](https://github.com/anishukserg/dacc/tree/master/crates/dacc-journal)), скан ([dacc-scan](https://github.com/anishukserg/dacc/tree/master/crates/dacc-scan)), экспорт ([dacc-access](https://github.com/anishukserg/dacc/tree/master/crates/dacc-access)).
