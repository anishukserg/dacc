use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-ls-show",
    title: NonEmptyStr::new("Перечень и запись: cargo dacc ls и show"),
    slice: crate::slice::s_access_contract,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("cargo dacc ls <kind> [--status <status>] перечисляет записи реестра с состоянием или статусом; cargo dacc show <id> отдаёт запись целиком; обе команды отвечают текстом и json; незнакомый вид или идентификатор — законный отказ с кодом причины."),
);
