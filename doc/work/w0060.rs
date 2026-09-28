use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(60,
    title: NonEmptyStr::new("Переименование slipway → dacc"),
    slice: crate::slice::s0020,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_029),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Крейты, идентификаторы, бинарь cargo-dacc, настройка dacc.toml и проза переименованы из slipway в dacc; workspace зелёный; старые slipway-* на crates.io — yank."
    ),
);
