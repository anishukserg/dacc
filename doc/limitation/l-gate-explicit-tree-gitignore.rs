use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-gate-explicit-tree-gitignore",
    title: NonEmptyStr::new("Явное дерево калитки уважает .gitignore"),
    violated: NonEmptyStr::new(
        "Рабочее дерево калитка обходит по списку git и .gitignore уважает, а явное дерево (gate --repo <root> <dir>) обходит вручную, пропуская только target/ и .git/: игнорируемые файлы в нём попадают в шаги внешних имён и ссылок (решение 8)."
    ),
);
