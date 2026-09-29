use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-release-0-3-0",
    title: NonEmptyStr::new("Выпуск 0.3.0"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Версия рабочего пространства поднята до 0.3.0 (minor: новые возможности — единый валидатор дат, ограничение Toil радиусом Local, погашение обязательств на сборке), тег v0.3.0 поставлен на коммит с ней, публикуемые крейты опубликованы 0.3.0 в порядке зависимостей; cargo search показывает 0.3.0."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
