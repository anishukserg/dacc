use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-first-commit",
    title: NonEmptyStr::new("Первый коммит в пустом репозитории делается инструментом"),
    violated: NonEmptyStr::new(
        "Правила коммита исполнимы самим инструментом с первого коммита: обходить их отдельным git add не требуется (решение 8)."
    ),
);
