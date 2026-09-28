use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-minimal-work-layer",
    title: NonEmptyStr::new("Минимальный слой работы и план DACC в нём"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "План DACC записан направлениями, срезами и единицами работы; ссылка на несуществующее решение, спецификацию или срез не компилируется; радиус работы выше потолка среза не компилируется."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
