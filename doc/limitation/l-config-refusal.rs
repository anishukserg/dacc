use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-config-refusal",
    title: NonEmptyStr::new("Отказ настройки называет форму числа; диапазон покрывает корневой коммит"),
    violated: NonEmptyStr::new(
        "Отказ называет, как его исправить, а проверка оснований покрывает каждый коммит на пути в основную ветку, включая корневой (решения 8 и 18)."
    ),
);
