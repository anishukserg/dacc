use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-gate-bypass",
    title: NonEmptyStr::new("Гейт закрытия читает запись подстроками и обходится формой"),
    violated: NonEmptyStr::new(
        "check_contract ищет подстроки в тексте записи: пустой якорь оформляется мультилайном, статус Enforced попадает в комментарий или rationale, перенос пути в specification ломает поиск; cap_json_map вставляет поле в готовый текст и связан с порядком ключей сериализатора."
    ),
);
