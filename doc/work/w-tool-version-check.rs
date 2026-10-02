use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-tool-version-check",
    title: NonEmptyStr::new("Инструмент отказывает работать вразнос с деревом"),
    slice: crate::slice::s_pilot_tool_findings,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_tool_version_drift,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("Мутирующие команды и калитка сверяют CARGO_PKG_VERSION бинарника с версией dacc-work в Cargo.lock дерева и отказывают кодом tool-version-drift с обеими версиями и шагом исправления; дерево без Cargo.lock и read-only команды слоя доступа не отказывают; сценарий падал до починки"),
);
