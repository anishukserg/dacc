# dacc-derive

Атрибут-маркер `#[doc_anchor]` [DACC](https://github.com/anishukserg/dacc): помечает фрагмент кода идентификатором, на который ссылаются решения.

```rust
#[dacc_derive::doc_anchor(id = "plan-ir", mode = "snippet")]
pub struct PlanIr {
    pub steps: Vec<u32>,
}
```

- **Элемент возвращается без изменений.** Константы ссылок порождает скан исходников ([dacc-scan](https://github.com/anishukserg/dacc/tree/master/crates/dacc-scan)), поэтому разметку можно ставить на любой элемент, допускающий атрибут, включая методы внутри `impl`.
- **Аргументы проверяются по тем же правилам, что у скана.** Опечатка в ключе или неизвестный режим — ошибка компиляции, а не молча пропавшая разметка.
- **Без `syn`.** Атрибут стоит в продуктовом коде, и его сборка не тянет парсер.

Лицензия — [Apache-2.0](https://github.com/anishukserg/dacc/blob/master/LICENSE).
