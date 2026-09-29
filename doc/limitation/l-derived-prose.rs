use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-derived-prose",
    title: NonEmptyStr::new("Выводимые числа пишутся руками, а не считаются"),
    violated: NonEmptyStr::new(
        "Порядковые номера, статусы и счётчики пилота считаются вручную через git log и grep и могут разойтись с реестром; гейт это расхождение не ловит."
    ),
);
