use dacc_core::NonEmptyStr;

dacc_work::upgrade!("u-0-4-0-journal-toml",
    from: NonEmptyStr::new("0.3.0"),
    to: NonEmptyStr::new("0.4.0"),
    subject: NonEmptyStr::new("the journal is one append-only record"),
    how: NonEmptyStr::new(
        "no migration: journal.toml is an [[events]] array, journal/ stays frozen history"
    ),
);
