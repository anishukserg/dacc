use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-attack-floor",
    title: NonEmptyStr::new("Пол измеряет исполненные атаки, а не любые doctest"),
    violated: NonEmptyStr::new(
        "Пол атак измеряет исполненные атаки, а не любые прошедшие примеры: иначе число добирается обычными doctest, и обещание «атаки не удалены» держится доброй волей (решение 12)."
    ),
);
