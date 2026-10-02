use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-agent-robustness",
    title: NonEmptyStr::new("Машинный контракт отказов и устойчивость параллельной работы"),
    thrust: crate::thrust::t_contract_discipline,
    outcome: NonEmptyStr::new(
        "Отказ каждой команды отвечает машиночитаемо в json и отвергает неподдерживаемый формат кодом причины, help объявляет все форматы; допуск задач пропускает дефектную запись и берёт следующую; разбор снимает вложенные комментарии и читает escape-последовательности, упоминание в комментарии не связывает записи; откат события возвращает и файл, и индекс git, лок коротко ждёт перед отказом; baseline держит точный размер долга."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
