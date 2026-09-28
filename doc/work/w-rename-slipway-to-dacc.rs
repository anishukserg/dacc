use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-rename-slipway-to-dacc",
    title: NonEmptyStr::new("Переименование slipway → dacc"),
    slice: crate::slice::s_rename_to_dacc,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_029),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Крейты, идентификаторы, бинарь cargo-dacc, настройка dacc.toml и проза переименованы из slipway в dacc; workspace зелёный; старые slipway-* на crates.io — yank."
    ),
);
