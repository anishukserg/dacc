use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use crate::taxonomy::Subsystem;
use dacc_work::WorkOrigin;

dacc_work::work!(4,
    title: NonEmptyStr::new("Генератор `work new` и замер трения заведения задач"),
    slice: crate::slice::s0002,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_005),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "`work new` пишет файл единицы работы по аргументам; по первым десяти единицам записаны время заведения и число циклов сборки; вердикт вынесен по критерию решения 5."
    ),
);
