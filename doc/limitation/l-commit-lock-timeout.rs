use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-commit-lock-timeout",
    title: NonEmptyStr::new("Блокировка коммита без номера процесса ждала до тайм-аута"),
    violated: NonEmptyStr::new(
        "Параллельный коммит ждёт блокировку, а блокировку умершего процесса снимает ожидающий (решения 8 и 14)."
    ),
);
