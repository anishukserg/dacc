use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-contract-assure",
    title: NonEmptyStr::new("Модуль contract: внутренние постусловия отделены от валидации"),
    slice: crate::slice::s_contract_enforcers,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("Модуль contract в dacc-core с макросом внутреннего постусловия: нарушение — паника с контекстом утверждения; документация модуля называет границу уровней — валидация ввода и среды остаётся Result с Refusal, panic там запрещён конвенцией модуля; паника на внутреннем нарушении покрыта тестом."),
);
