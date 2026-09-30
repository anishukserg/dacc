use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-knowledge-layer-boundary",
    title: NonEmptyStr::new("Записи знания не в слое работы"),
    violated: NonEmptyStr::new(
        "Типы invariant! и review! — записи знания (гарантия и чтение), но лежали в dacc-work; их место в dacc-knowledge рядом с adr! и rfc!."
    ),
);
