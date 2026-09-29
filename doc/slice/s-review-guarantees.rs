use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-review-guarantees",
    title: NonEmptyStr::new("Гарантии, объявленные, но не проверенные на сборке"),
    thrust: crate::thrust::t_honest_guarantees,
    outcome: NonEmptyStr::new(
        "Правило, объявленное в реестре с силой «компилятор», действительно проверяется компилятором: журнал отвергает несуществующие даты тем же валидатором, что и макрос date!; обязательство погашено только приземлённой работой с WorkOrigin::Obligation. Дефекты инструмента закрыты: work new пишет замер в настроенный каталог и откатывает файл при отказе."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
