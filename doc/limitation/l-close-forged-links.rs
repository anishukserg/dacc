use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-close-forged-links",
    title: NonEmptyStr::new("w-close-forged-links"),
    violated: NonEmptyStr::new(
        "Ссылка на живое решение из позиции «замещённое» не компилируется."
    ),
);
