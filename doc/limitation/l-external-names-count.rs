use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-external-names-count",
    title: NonEmptyStr::new("Пустой список внешних имён не заявляет шаг выполненным"),
    violated: NonEmptyStr::new(
        "Калитка с пустым списком внешних имён пропускает шаг, но общий счёт заявляет полное число шагов: вердикт обещает больше, чем исполнил."
    ),
);
