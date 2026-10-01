use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-access-modules",
    title: NonEmptyStr::new("Слой доступа по модулям и бюджет размера файла"),
    thrust: crate::thrust::t_access_layer,
    outcome: NonEmptyStr::new(
        "Ответы инструмента говорят факт: счётчики всех стадий, честное усечение с shown и total, разметка из кода; access.rs разобран на модули по командам с одним разбором аргументов и одной формой строк; размер каждого файла исходника держит тест калитки — известный долг назван именованным baseline и только уменьшается."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
