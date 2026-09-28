# dacc-knowledge

Слой знания [DACC](https://github.com/anishukserg/dacc): схема архитектурных решений и доменных спецификаций, макросы регистрации `adr!` и `rfc!`.

Документ — файл реестра на Rust, по файлу на документ. Правила, которые обычно проверяет валидатор, выражены типами:

- ломающее решение без инструкций миграции невыразимо — инструкции живут внутри варианта `Breaking::Yes`;
- замещение без указания замещающего решения невыразимо — `DocStatus::SupersededBy` несёт ссылку;
- решение без авторов и спецификация без инвариантов не компилируются.

Модуль `attacks` — атакующие doctest `compile_fail` с контролями: каждое обещание «не компилируется» проверено попыткой его нарушить.

Константы-ссылки на документы порождает [dacc-scan](https://github.com/anishukserg/dacc/tree/master/crates/dacc-scan) из `build.rs` крейта документов. Пример реестра — [examples/demo-doc](https://github.com/anishukserg/dacc/tree/master/examples/demo-doc).

Лицензия — [Apache-2.0](https://github.com/anishukserg/dacc/blob/master/LICENSE).
