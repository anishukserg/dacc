use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-legitimate-failure",
    title: NonEmptyStr::new("Сбой отвечает legitimate: false"),
    slice: crate::slice::s_gate_integrity,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("Паника в контексте --format json перехватывается хуком и печатает dacc-error с legitimate: false и текстом нарушения: машинный контракт отличает сбой от законного отказа в любом режиме; тест вызывает внутреннее нарушение и проверяет форму ответа."),
);
