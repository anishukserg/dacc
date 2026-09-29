use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-single-journal-code",
    title: NonEmptyStr::new("Реализовать: журнал одной append-only записью"),
    slice: crate::slice::s_compact_ceremony_code,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_043),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Журнал — doc/journal.toml, массив таблиц событий; события дописываются, калитка проверяет «прежняя версия — префикс новой»; свёртка читает одну запись."
    ),
);
