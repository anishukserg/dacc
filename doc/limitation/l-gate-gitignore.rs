use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-gate-gitignore",
    title: NonEmptyStr::new("Обход дерева в калитке уважает .gitignore"),
    violated: NonEmptyStr::new(
        "Калитка обходит дерево, пропуская только каталоги target/ и .git/: игнорируемые .gitignore пути (.venv-docs с site-packages) и чужой битый license.txt ломают шаги внешних имён и ссылок, хотя в репозиторий не входят (решение 8)."
    ),
);
