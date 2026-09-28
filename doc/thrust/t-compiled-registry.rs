use dacc_core::NonEmptyStr;

dacc_work::thrust!("t-compiled-registry",
    title: NonEmptyStr::new("Спецификация методологии — компилируемый реестр"),
    outcome: NonEmptyStr::new(
        "Нормативный текст методологии живёт в реестре doc/; markdown спецификации порождается рендером, и ручная правка порождённого текста отвергается проверкой свежести."
    ),
);
