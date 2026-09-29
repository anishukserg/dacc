use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-scan-english-errors",
    title: NonEmptyStr::new("Порождённый сканом текст и его ошибки — на английском"),
    violated: NonEmptyStr::new(
        "Текст, порождаемый инструментом, английский, включая комментарии и документацию порождённых файлов и сообщения, которые они выдают компилятором (решение 22)."
    ),
);
