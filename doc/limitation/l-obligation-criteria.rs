use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-obligation-criteria",
    title: NonEmptyStr::new("Критерии погашения обязательства — обязательны и типизированы"),
    violated: NonEmptyStr::new(
        "Obligation несёт только discharged_when (свободный текст): вопрос погашается «когда-нибудь», критерии решения не названы."
    ),
);
