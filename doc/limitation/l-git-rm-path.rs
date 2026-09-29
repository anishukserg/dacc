use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-git-rm-path",
    title: NonEmptyStr::new("Путь, удалённый через git rm, не коммитился"),
    violated: NonEmptyStr::new(
        "Пути перечисляются явно — новые, изменённые и удалённые; коммитятся ровно они (решение 8)."
    ),
);
