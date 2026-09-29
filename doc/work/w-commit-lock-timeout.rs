use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-commit-lock-timeout",
    title: NonEmptyStr::new("Блокировка коммита без номера процесса ждала до тайм-аута"),
    slice: crate::slice::s_cargo_dacc_tool,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_commit_lock_timeout,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Сценарий, падавший до починки, проходит: пустой файл блокировки старше порога — след flock прежнего скрипта или сбой между созданием файла и записью номера — снимается; свежий файл без номера уважается; отказ по тайм-ауту называет держателя."
    ),
);
