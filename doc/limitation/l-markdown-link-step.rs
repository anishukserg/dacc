use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-markdown-link-step",
    title: NonEmptyStr::new("Шаг ссылок в markdown не выполнялся на дереве коммита"),
    violated: NonEmptyStr::new(
        "Калитка на дереве коммита проверяет относительные ссылки в markdown (решение 8)."
    ),
);
