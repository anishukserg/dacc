use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-publish-knowledge-crates",
    title: NonEmptyStr::new("Публикация крейтов знания в crates.io"),
    slice: crate::slice::s_publication,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_025),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Пять крейтов готовы к публикации одной версией 0.1.0: core и journal проходят cargo package --locked целиком; knowledge, derive и scan собраны в пакеты, их зависимости разрешаются после публикации core и journal. Порядок публикации и версия закреплены; сама публикация — с токеном."
    ),
);
