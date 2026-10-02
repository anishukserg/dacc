use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-machine-refusals",
    title: NonEmptyStr::new("Отказ инструмента рвал машинный контракт"),
    violated: NonEmptyStr::new(
        "Отказ отвечает машиночитаемо в каждом режиме: --format json получает dacc-error с полем legitimate, xml-формат либо поддержан, либо отвергнут кодом причины, и контракт help объявляет каждый поддерживаемый формат."
    ),
);
