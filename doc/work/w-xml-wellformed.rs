use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-xml-wellformed",
    title: NonEmptyStr::new("XML без запрещённых символов"),
    slice: crate::slice::s_gate_integrity,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("xml_string заменяет управляющие символы XML 1.0 пробелом — как вывод журнала, — и экранирует апостроф; тест прогоняет управляющий символ через map и where в xml и проверяет ответ парсером-минималистом."),
);
