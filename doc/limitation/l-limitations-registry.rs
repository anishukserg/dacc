use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-limitations-registry",
    title: NonEmptyStr::new("Реестр известных ограничений со сверкой кода"),
    violated: NonEmptyStr::new(
        "Работы с происхождением Divergence называют расхождение текстом, но реестр не сверяется с кодом: необъявленное расхождение не ловится сборкой."
    ),
);
