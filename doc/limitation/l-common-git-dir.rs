use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-common-git-dir",
    title: NonEmptyStr::new("Доказательство и список имён — от общего каталога git"),
    violated: NonEmptyStr::new(
        "Доказательство готовности принадлежит репозиторию: калитка, пройденная в изолированной рабочей копии, остаётся доказательством того же дерева (решения 4 и 15)."
    ),
);
