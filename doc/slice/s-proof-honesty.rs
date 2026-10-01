use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-proof-honesty",
    title: NonEmptyStr::new("Доказательство проверяемо, а не заявлено"),
    thrust: crate::thrust::t_contract_discipline,
    outcome: NonEmptyStr::new(
        "RedBefore называет существующий в дереве и исполняемый калиткой тест — вымышленное имя отвергается кодом причины; статус Enforced требует непустого tests с якорем на исполняемый тест, и i-work-origin переведён на такой якорь; остаточная граница «падение до починки — заявление» названа ограничением l-red-before-claim."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
