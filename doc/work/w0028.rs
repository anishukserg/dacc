use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(28,
    title: NonEmptyStr::new("Лицензия, README и метаданные публикации"),
    slice: crate::slice::s0009,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_017),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "В корне — LICENSE с текстом Apache-2.0 и README.md; у каждого из семи публикуемых крейтов — LICENSE, README.md и поля license, repository, readme, keywords и categories; калитка проходит."
    ),
);
