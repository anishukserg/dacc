use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-machine-refusals",
    title: NonEmptyStr::new("Отказы работ отвечают машинному контракту"),
    slice: crate::slice::s_agent_robustness,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_machine_refusals,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Отказ work-команд в --format json печатает dacc-error с полем legitimate; work next --format xml отвергается кодом format-invalid как в слое доступа; help объявляет json и xml у map и where; допуск work next пропускает запись без таксона и берёт следующую, а не встаёт всей очередью; каждый пункт доказан тестом."
    ),
);
