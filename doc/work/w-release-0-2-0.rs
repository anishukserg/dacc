use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-release-0-2-0",
    title: NonEmptyStr::new("Выпуск 0.2.0: версия, тег v0.2.0, публикация"),
    slice: crate::slice::s_release_0_2_0,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_030),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Версия рабочего пространства поднята до 0.2.0, тег v0.2.0 поставлен на коммит с ней, публикуемые крейты опубликованы 0.2.0 в порядке зависимостей; cargo search показывает 0.2.0."
    ),
);
