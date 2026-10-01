use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-access-contract",
    title: NonEmptyStr::new("Контракт RFC-0003 доделан: state, ls, show, refs, find, brief, help"),
    thrust: crate::thrust::t_access_layer,
    outcome: NonEmptyStr::new(
        "Команды state, ls, show, refs, find, brief и help отвечают из реестра; help --format json объявляет стоимость и мутирование каждой команды; ответ map не превышает 8 КБ, state — 2 КБ и несёт только изменяющееся за день; усечение явное; законный отказ в json отличается от сбоя полем legitimate; xml доступен map и where наравне с json."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
