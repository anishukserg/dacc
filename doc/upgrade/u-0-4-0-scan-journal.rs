use dacc_core::NonEmptyStr;

dacc_work::upgrade!("u-0-4-0-scan-journal",
    from: NonEmptyStr::new("0.3.0"),
    to: NonEmptyStr::new("0.4.0"),
    subject: NonEmptyStr::new("scan_journal now takes two paths"),
    how: NonEmptyStr::new(
        "pass journal.toml and journal/ as two paths in build.rs instead of one directory"
    ),
);
