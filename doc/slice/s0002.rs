use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(2,
    title: NonEmptyStr::new("Минимальный слой работы и план DACC в нём"),
    thrust: crate::thrust::t0002,
    outcome: NonEmptyStr::new(
        "План DACC записан направлениями, срезами и единицами работы; ссылка на несуществующее решение, спецификацию или срез не компилируется; радиус работы выше потолка среза не компилируется."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
