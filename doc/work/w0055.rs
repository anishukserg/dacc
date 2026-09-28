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
        "Крейты slipway-core, slipway-knowledge, slipway-scan, slipway-derive опубликованы в crates.io одной версией 0.1.0; cargo package --locked и cargo publish --dry-run проходят; в манифесте пилота крейты закреплены по версии."
    ),
);
