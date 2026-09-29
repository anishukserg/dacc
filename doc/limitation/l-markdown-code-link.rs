use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-markdown-code-link",
    title: NonEmptyStr::new("Шаг ссылок в markdown принимал код за ссылку"),
    violated: NonEmptyStr::new(
        "Проверка содержательна: шаг калитки отвергает документ за настоящую битую ссылку, а не за форму записи (решения 8 и 14)."
    ),
);
