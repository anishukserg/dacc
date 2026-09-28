use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(32,
    title: NonEmptyStr::new("Текст инструмента на английском"),
    slice: crate::slice::s0012,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_019),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Вывод инструмента, темы и тела создаваемых им коммитов и комментарии порождённых хуков — английские; отказ не называет номер решения чужого реестра; сценарии и документы DACC обновлены под новые строки."
    ),
);
