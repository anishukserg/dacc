use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-access-first-touch",
    title: NonEmptyStr::new("Первое касание: map, where и директивы агентов"),
    thrust: crate::thrust::t_access_layer,
    outcome: NonEmptyStr::new(
        "cargo dacc map печатает карту реестра со свёрткой журнала и счётчиками; cargo dacc where --file называет разметку файла, ссылающиеся документы и связанные работы; cargo dacc emit all пишет файлы-директивы агентов, и повторный запуск идентичен закоммиченному; map и where отвечают текстом и JSON."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
