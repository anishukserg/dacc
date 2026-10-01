use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-gate-integrity",
    title: NonEmptyStr::new("Гейт не обходится формой записи"),
    thrust: crate::thrust::t_contract_discipline,
    outcome: NonEmptyStr::new(
        "Записи реестра читаются значениями полей, а не подстроками: комментарии, переносы и мультилайн не обходят проверку контракта — атаки на все три формы отвергаются; усечение ответов происходит до сериализации и не ломает json; управляющие символы XML заменяются до вывода; сбой в json отвечает legitimate: false; запись журнала защищена межпроцессным локом."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
