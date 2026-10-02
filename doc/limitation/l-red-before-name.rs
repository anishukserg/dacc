use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-red-before-name",
    title: NonEmptyStr::new("RedBefore называл функцию, а не тест"),
    violated: NonEmptyStr::new(
        "RedBefore доказывает тест: имя точно совпадает с именем функции, и на ней стоит атрибут теста — префикс имени и обычная функция не принимаются"
    ),
);
