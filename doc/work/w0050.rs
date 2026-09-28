use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(50,
    title: NonEmptyStr::new("Установка хуков не оставляет частичную установку"),
    slice: crate::slice::s0011,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "Установка хуков ставит все правила или отказывает: частичная установка, выглядящая успешной, оставляет основание коммита непроверенным (решения 8 и 14)."
        ),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Сценарий, падавший до починки, проходит: чужой хук в каталоге останавливает установку до первой записи ненулевым кодом с именем файла и подсказкой про делегирование проверки сообщения; свой хук распознаётся по маркеру и переписывается; замена чужого — только по явному флагу."
    ),
);
