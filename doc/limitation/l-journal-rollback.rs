use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-journal-rollback",
    title: NonEmptyStr::new("Сбой коммита журнала оставлял след в индексе"),
    violated: NonEmptyStr::new(
        "Запись журнала транзакционна: отказ коммита возвращает и файл, и индекс git в прежнее состояние, а конкуренция за лок коротко ждёт и только потом отказывает кодом причины."
    ),
);
