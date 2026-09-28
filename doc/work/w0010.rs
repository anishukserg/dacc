use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use crate::taxonomy::Subsystem;
use dacc_work::WorkOrigin;

dacc_work::work!(10,
    title: NonEmptyStr::new("Документы реестров без каталога src"),
    slice: crate::slice::s0003,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_010),
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Решения, спецификации, план и демо-реестр открываются из корня своих крейтов; сборка, тесты и проверка сообщения коммита работают по новым путям."
    ),
);
