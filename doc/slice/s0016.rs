use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(16,
    title: NonEmptyStr::new("Порождённый сканом текст — английский"),
    thrust: crate::thrust::t0002,
    outcome: NonEmptyStr::new(
        "Комментарии, документация и сообщения об ошибках, которые скан порождает в чужом крейте документов, английские, как и остальной текст инструмента; русским остаётся только то, что написал сам продукт."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
