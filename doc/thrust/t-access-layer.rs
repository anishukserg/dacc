use dacc_core::NonEmptyStr;

dacc_work::thrust!("t-access-layer",
    title: NonEmptyStr::new("Человек и агент узнают всё о месте одной командой"),
    outcome: NonEmptyStr::new(
        "Команды слоя доступа отвечают из реестра текстом и машинным форматом, и агент не прибегает к find, grep и ls для навигации по решениям, спецификациям, задачам, долгу и владельцам."
    ),
);
