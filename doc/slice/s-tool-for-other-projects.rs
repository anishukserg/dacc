use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-tool-for-other-projects",
    title: NonEmptyStr::new("Инструмент под чужой проект: язык и настройка"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Инструмент говорит и пишет по-английски, а раскладку реестра, правила сообщения, шаблоны служебных тем, пол атак и собственную проверку проект задаёт настройкой; сценарий на чужой раскладке проходит целиком."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
