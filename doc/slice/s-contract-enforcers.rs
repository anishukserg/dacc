use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-contract-enforcers",
    title: NonEmptyStr::new("Контрактная дисциплина: assure!, связь спецификаций и законы форматов"),
    thrust: crate::thrust::t_contract_discipline,
    outcome: NonEmptyStr::new(
        "Модуль contract в dacc-core несёт внутренние постусловия паникой с контекстом, и граница L3/L4 названа: валидация ввода и среды остаётся Refusal с legitimate; запись инварианта типизированно связана со своей спецификацией, и закрытие среза требует Enforced с живым якорем enforced_by; законы форматов именованы и покрыты тестами без новых зависимостей."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
