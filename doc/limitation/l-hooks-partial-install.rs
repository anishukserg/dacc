use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-hooks-partial-install",
    title: NonEmptyStr::new("Установка хуков не оставляет частичную установку"),
    violated: NonEmptyStr::new(
        "Установка хуков ставит все правила или отказывает: частичная установка, выглядящая успешной, оставляет основание коммита непроверенным (решения 8 и 14)."
    ),
);
