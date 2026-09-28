use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-generated-lints",
    title: NonEmptyStr::new("Порождённый код и макросы регистрации под строгим профилем lints"),
    slice: crate::slice::s_pilot_blockers,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_016),
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Реестры doc/ и examples/demo-doc объявляют профиль решения 16 на уровне forbid и проходят калитку; до починки тот же профиль давал отказ clippy на порождённом коде; каждый публичный элемент порождённого кода и макросов регистрации документирован."
    ),
);
