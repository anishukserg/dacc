use dacc_core::NonEmptyStr;

dacc_work::thrust!("t-borrow-from-angarabase",
    title: NonEmptyStr::new("Заимствовать механизмы AngaraBase в DACC — типом, а не скриптом"),
    outcome: NonEmptyStr::new(
        "Каждый заимствованный у AngaraBase механизм выражен в DACC типом или compile-time-проверкой, а не runtime-скриптом поверх markdown; что нельзя выразить типом, не заимствуется. Кандидаты: наблюдаемые гарантии, ратчет миграции проверок, anti-vacuity, реестр ограничений, контрактные границы, критерии решений."
    ),
);
