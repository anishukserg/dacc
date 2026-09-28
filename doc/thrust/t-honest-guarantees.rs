use dacc_core::NonEmptyStr;

dacc_work::thrust!("t-honest-guarantees",
    title: NonEmptyStr::new("Гарантии DACC честны и проверены атакой"),
    outcome: NonEmptyStr::new(
        "Каждое обещание GUARANTEES.md с силой «невозможно» или «компилятор» имеет атакующий compile_fail-тест с позитивным контролем, и набор атак зелёный на основной ветке."
    ),
);
