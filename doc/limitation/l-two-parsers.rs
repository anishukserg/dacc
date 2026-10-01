use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-two-parsers",
    title: NonEmptyStr::new("Один реестр читали два разбора и три экранировщика"),
    violated: NonEmptyStr::new(
        "Запись реестра читается значениями полей в одном месте: подстрочный разбор и разрозненные экранировщики JSON не дают форме ответа зависеть от места чтения."
    ),
);
