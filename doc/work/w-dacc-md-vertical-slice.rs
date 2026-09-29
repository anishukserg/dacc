use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-dacc-md-vertical-slice",
    title: NonEmptyStr::new("Вертикальный срез в DACC.md"),
    slice: crate::slice::s_dacc_md,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "DACC.md не объясняет, что работа организована вертикальными срезами через слои, а не горизонтальными слоями."
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "DACC.md описывает вертикальный срез: направление, срез и единицу работы как одно вертикальное изменение через все слои, закрываемое по исходу, а не по календарю."
    ),
);
