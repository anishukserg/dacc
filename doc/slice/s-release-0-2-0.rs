use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-release-0-2-0",
    title: NonEmptyStr::new("Выпуск 0.2.0"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Версия рабочего пространства поднята до 0.2.0 (minor: новые возможности), тег v0.2.0 поставлен на коммит с ней, публикуемые крейты опубликованы 0.2.0 в порядке зависимостей; cargo search показывает 0.2.0."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
