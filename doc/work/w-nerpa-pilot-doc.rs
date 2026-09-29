use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-nerpa-pilot-doc",
    title: NonEmptyStr::new("Записать пилот nerpa в DACC.md и объявить зазор"),
    slice: crate::slice::s_nerpa_pilot_doc,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Пилот nerpa (волна 3) прошёл слой знания: компиляция ловит переименования и удаления, но не семантические расхождения; зазор «пределы проверок наблюдаемы» не закрыт ни одним срезом."
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "DACC.md: реестр «поймано/пропущено» в «Честные границы» и строка версии 1.4; объявлен срез s-observable-check-limits (пределы проверок наблюдаемы)."
    ),
);
