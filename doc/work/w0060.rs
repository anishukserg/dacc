use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(60,
    title: NonEmptyStr::new("Переименование dacc → dacc"),
    slice: crate::slice::s0020,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_029),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Крейты, идентификаторы, бинарь cargo-dacc, настройка dacc.toml и проза переименованы из dacc в dacc; workspace зелёный; старые dacc-* на crates.io — yank."
    ),
);
