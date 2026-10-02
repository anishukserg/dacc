use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-release-0-5-0",
    title: NonEmptyStr::new("Выпуск 0.5.0: версия, тег v0.5.0, публикация"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Версия рабочего пространства поднята до 0.5.0 вместе с версиями path-зависимостей; тег v0.5.0 поставлен на коммит с ней; публикуемые крейты опубликованы 0.5.0 на crates.io в порядке зависимостей; cargo search показывает 0.5.0."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
