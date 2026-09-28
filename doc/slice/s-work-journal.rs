use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-work-journal",
    title: NonEmptyStr::new("Журнал работы: события, доказательство готовности, команды"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Состояние каждой работы и среза DACC вычисляется свёрткой doc/journal при сборке; приземление без доказательства на том же дереве не собирается; события пишут команды cargo dacc work и slice; прошлые работы восстановлены из истории."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
