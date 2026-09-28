use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-scan-english",
    title: NonEmptyStr::new("Порождённый сканом текст — английский"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Комментарии, документация и сообщения об ошибках, которые скан порождает в чужом крейте документов, английские, как и остальной текст инструмента; русским остаётся только то, что написал сам продукт."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
