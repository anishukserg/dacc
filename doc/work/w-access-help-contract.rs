use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-help-contract",
    title: NonEmptyStr::new("Машинный контракт: help --format json и поле legitimate"),
    slice: crate::slice::s_access_contract,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("cargo dacc help --format json перечисляет команды с их стоимостью и мутированием; отказ команды слоя доступа в json несёт поле legitimate и код причины и отличается от сбоя; тест сверяет контракт для каждой команды."),
);
