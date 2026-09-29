use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-single-form",
    title: NonEmptyStr::new("Один дом для формы правила, дубли запрещены"),
    violated: NonEmptyStr::new(
        "anchor_rules — «одно место на всех исполнителей», но механизма, который ловит копии формы правила, нет: дубль парсера не ловится."
    ),
);
