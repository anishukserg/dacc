use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-anti-vacuity",
    title: NonEmptyStr::new("Anti-vacuity: скан-паттерн краснеет, когда перестал матчиться"),
    violated: NonEmptyStr::new(
        "Скан-паттерны DACC (например, поиск обхода __from_scan) могут протухнуть молча: ноль сырых совпадений выглядит как чистое дерево, а не как умершая проверка."
    ),
);
