use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-json-verdicts",
    title: NonEmptyStr::new("Вердикты и отказы в JSON"),
    slice: crate::slice::s_machine_contract,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_022),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Калитка, коммит, проверка сообщения и состояние работы принимают --format json и печатают вердикт полями; человекочитаемый вывод остаётся по умолчанию; сценарии закрепляют снимок каждой формы."
    ),
);
