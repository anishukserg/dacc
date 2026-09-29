use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-facade-crate",
    title: NonEmptyStr::new("Фасадный крейт — публичная поверхность"),
    violated: NonEmptyStr::new(
        "Решение 7 называет публичной поверхностью фасадный крейт и cargo-dacc, но фасадного крейта в репозитории нет: потребитель подключает крейты по одному."
    ),
);
