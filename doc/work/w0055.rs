use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(55,
    title: NonEmptyStr::new("Публикация крейтов знания в crates.io"),
    slice: crate::slice::s0018,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_025),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Пять крейтов готовы к публикации одной версией 0.1.0: core и journal проходят cargo package --locked целиком; knowledge, derive и scan собраны в пакеты, их зависимости разрешаются после публикации core и journal. Порядок публикации и версия закреплены; сама публикация — с токеном."
    ),
);
