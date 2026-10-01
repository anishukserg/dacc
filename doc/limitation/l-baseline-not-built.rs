use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-baseline-not-built",
    title: NonEmptyStr::new("Ратчет объявлен решением 40, механизм не построен"),
    violated: NonEmptyStr::new(
        "Строгая проверка вводится вместе с механизмом declare_baseline!: известный долг заморожен и назван, блокируется только его рост, а baseline уменьшается отдельным решением (решение 40)."
    ),
);
