use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-external-names-count",
    title: NonEmptyStr::new("Шаг внешних имён честно в счёте"),
    slice: crate::slice::s_measurable,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_external_names_count,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Пустой список внешних имён не заявляет шаг выполненным: вердикт не обещает полного счёта, когда шаг не выполнялся."
    ),
);
