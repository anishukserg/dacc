use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-squash-events-code",
    title: NonEmptyStr::new("Реализовать: started/gate/landed одним коммитом"),
    slice: crate::slice::s_compact_ceremony_code,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_042),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "work land пишет started, gate и landed в одном коммите; work start больше не пишет отдельный коммит; автомат свёртки сохранён."
    ),
);
