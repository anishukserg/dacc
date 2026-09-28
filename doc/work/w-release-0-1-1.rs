use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-release-0-1-1",
    title: NonEmptyStr::new("Выпуск 0.1.1: версия, тег v0.1.1, публикация"),
    slice: crate::slice::s_versioning,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_030),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Версия рабочего пространства поднята до 0.1.1, тег v0.1.1 поставлен на коммит с ней, публикуемые крейты опубликованы 0.1.1 в порядке зависимостей; cargo search показывает 0.1.1."
    ),
);
