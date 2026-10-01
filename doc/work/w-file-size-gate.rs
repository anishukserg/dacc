use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-file-size-gate",
    title: NonEmptyStr::new("Размер файла держит тест калитки с именованным baseline"),
    slice: crate::slice::s_access_modules,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_040),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Тест, который гоняет калитка на каждом дереве, держит бюджет строк файла исходника: файл сверх бюджета назван baseline с замороженным размером, рост сверх него — отказ теста, и baseline только уменьшается отдельным решением."
    ),
);
