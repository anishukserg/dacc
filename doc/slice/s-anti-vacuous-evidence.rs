use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-anti-vacuous-evidence",
    title: NonEmptyStr::new("Анти-вакуумные виды доказательства выражены типом"),
    thrust: crate::thrust::t_borrow_from_angarabase,
    outcome: NonEmptyStr::new(
        "RedBefore, MutationProof и AntiVacuum — варианты типа доказательства с непустым предметом; событие landed хранит их плоскими полями и читает прежние события без правки; work land отвергает приземление дефекта (Divergence) или необратимого (Persistent/Irreversible) без RedBefore с кодом причины."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
