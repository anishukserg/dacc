use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-hooks-partial-install",
    title: NonEmptyStr::new("Установка хуков не оставляет частичную установку"),
    slice: crate::slice::s_tool_defects,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_hooks_partial_install,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Сценарий, падавший до починки, проходит: чужой хук в каталоге останавливает установку до первой записи ненулевым кодом с именем файла и подсказкой про делегирование проверки сообщения; свой хук распознаётся по маркеру и переписывается; замена чужого — только по явному флагу."
    ),
);
