use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-truth-of-now",
    title: NonEmptyStr::new("Контракт и проекция состояния не смешиваются"),
    violated: NonEmptyStr::new(
        "Проекции (roadmap, сайт) порождаются из реестра, но правило «проекция не хранит контракт» не выражено: проекция может начать дублировать нормативное."
    ),
);
