//! События журнала: предмет, время, вид и поля вида.

use crate::format::{self, Record};
use crate::time;

/// Предмет события: единица работы или срез, адресуемый slug.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subject {
    Work(String),
    Slice(String),
}

impl Subject {
    /// Идентификатор в плане: `w-fix-gitignore` или `s-versioning`.
    pub fn id(&self) -> &str {
        match self {
            Self::Work(slug) | Self::Slice(slug) => slug,
        }
    }

    /// Разбирает `w-fix-gitignore` или `s-versioning`.
    pub fn parse(text: &str) -> Option<Subject> {
        match text.as_bytes() {
            [b'w', b'-', ..] => Some(Self::Work(text.to_owned())),
            [b's', b'-', ..] => Some(Self::Slice(text.to_owned())),
            _ => None,
        }
    }
}

/// Чем подтверждено приземление.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evidence {
    /// Событием gate на том же дереве.
    Gate,
    /// Восстановлено из трейлеров истории, когда журнала ещё не было.
    History,
}

/// Вердикт калитки в событии gate (решение 22).
///
/// Журнал двуформатный: прежние события несут прозу, новые — структуру. Прозу
/// прежних событий журнал читает без правки — файл события неизменен.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateVerdict {
    /// Прозаический вердикт прежнего события (решение 19).
    Prose(String),
    /// Структурный вердикт нового события: пройденные шаги, их общее число,
    /// пропущенные, число прошедших атак и минимальный тулчейн. `msrv` есть
    /// только у полного яруса; ярус коммита минимальную версию не исполняет
    /// (решение 33), и `msrv` у него отсутствует.
    Structured {
        passed: usize,
        total: usize,
        skipped: Vec<String>,
        attacks: usize,
        msrv: Option<String>,
    },
}

/// Вид события и его поля.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Started,
    Gate {
        gate: String,
        tree: String,
        verdict: GateVerdict,
    },
    Landed {
        commit: String,
        tree: String,
        evidence: Evidence,
    },
    Abandoned {
        reason: String,
    },
    Closed,
}

impl Kind {
    /// Имя вида в файле и в имени файла.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Gate { .. } => "gate",
            Self::Landed { .. } => "landed",
            Self::Abandoned { .. } => "abandoned",
            Self::Closed => "closed",
        }
    }
}

/// Событие журнала.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// Путь файла от каталога журнала через `/` — имя события в сообщениях.
    pub file: String,
    pub subject: Subject,
    /// Время в UTC, `ГГГГ-ММ-ДДTЧЧ:ММ:ССZ`.
    pub at: String,
    pub kind: Kind,
}

impl Event {
    /// Новое событие с каноническим путём файла.
    pub fn new(subject: Subject, at: String, kind: Kind) -> Event {
        Event {
            file: relative_path(&subject, &at, &kind, 0),
            subject,
            at,
            kind,
        }
    }

    /// Событие из разобранного файла. Поля проверяются строго: лишнее,
    /// недостающее или неверно записанное поле — ошибка с названием поля.
    pub fn from_record(file: &str, record: &Record) -> Result<Event, String> {
        let name = record.get("event").ok_or("no event field")?;
        let (subject_key, own): (&str, &[&str]) = match name {
            "started" => ("work", &[]),
            "gate" => (
                "work",
                &[
                    "gate", "tree", "verdict", "passed", "total", "skipped", "attacks", "msrv",
                ],
            ),
            "landed" => ("work", &["commit", "tree", "evidence"]),
            "abandoned" => ("work", &["reason"]),
            "closed" => ("slice", &[]),
            other => {
                return Err(format!(
                    "unknown event `{other}`: started, gate, landed, abandoned, closed"
                ))
            }
        };
        for (key, _) in &record.fields {
            let known = key == "event" || key == "at" || key == subject_key;
            if !known && !own.contains(&key.as_str()) {
                return Err(format!("extra field `{key}` on a {name} event"));
            }
        }

        let subject_text = record
            .get(subject_key)
            .ok_or_else(|| format!("no {subject_key} field on a {name} event"))?;
        let subject = Subject::parse(subject_text)
            .filter(|subject| {
                matches!(
                    (subject, subject_key),
                    (Subject::Work(_), "work") | (Subject::Slice(_), "slice")
                )
            })
            .ok_or_else(|| {
                let example = if subject_key == "work" {
                    "w-fix-gitignore"
                } else {
                    "s-versioning"
                };
                format!("field {subject_key} is an identifier like {example}, not {subject_text:?}")
            })?;

        let at = record.get("at").ok_or("no at field")?;
        if !time::is_timestamp(at) {
            return Err(format!(
                "field at is a UTC time like 2026-09-11T03:15:00Z, not {at:?}"
            ));
        }

        let required = |key: &str| {
            record
                .get(key)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("no non-empty field {key} on a {name} event"))
        };
        let number = |key: &str| {
            let value = required(key)?;
            value
                .parse::<usize>()
                .map_err(|_| format!("field {key} is a non-negative number, not {value:?}"))
        };
        let hash = |key: &str| {
            let value = required(key)?;
            if is_hash(&value) {
                Ok(value)
            } else {
                Err(format!(
                    "field {key} is a git hash of 40 or 64 lowercase hexadecimal characters"
                ))
            }
        };
        let kind = match name {
            "started" => Kind::Started,
            "gate" => Kind::Gate {
                gate: required("gate")?,
                tree: hash("tree")?,
                verdict: match record.get("verdict") {
                    Some(verdict) => GateVerdict::Prose(verdict.to_owned()),
                    None => GateVerdict::Structured {
                        passed: number("passed")?,
                        total: number("total")?,
                        skipped: record.get("skipped").map_or_else(Vec::new, |skipped| {
                            skipped
                                .split(',')
                                .map(str::trim)
                                .filter(|step| !step.is_empty())
                                .map(str::to_owned)
                                .collect()
                        }),
                        attacks: number("attacks")?,
                        msrv: record.get("msrv").map(str::to_owned),
                    },
                },
            },
            "landed" => Kind::Landed {
                commit: hash("commit")?,
                tree: hash("tree")?,
                evidence: match record.get("evidence") {
                    None => Evidence::Gate,
                    Some("history") => Evidence::History,
                    Some(other) => {
                        return Err(format!("field evidence is only history, not {other:?}"))
                    }
                },
            },
            "abandoned" => Kind::Abandoned {
                reason: required("reason")?,
            },
            _ => Kind::Closed,
        };

        let event = Event {
            file: file.to_owned(),
            subject,
            at: at.to_owned(),
            kind,
        };
        if !event.file_matches() {
            return Err(format!(
                "the file name does not match the event: expected {}",
                relative_path(&event.subject, &event.at, &event.kind, 0)
            ));
        }
        Ok(event)
    }

    /// Текст файла события.
    pub fn to_text(&self) -> String {
        let subject_key = match self.subject {
            Subject::Work(_) => "work",
            Subject::Slice(_) => "slice",
        };
        let id = self.subject.id();
        let mut fields: Vec<(String, String)> = vec![
            ("event".to_owned(), self.kind.name().to_owned()),
            (subject_key.to_owned(), id.to_owned()),
            ("at".to_owned(), self.at.clone()),
        ];
        match &self.kind {
            Kind::Started | Kind::Closed => {}
            Kind::Gate {
                gate,
                tree,
                verdict,
            } => {
                fields.push(("gate".to_owned(), gate.clone()));
                fields.push(("tree".to_owned(), tree.clone()));
                match verdict {
                    GateVerdict::Prose(verdict) => {
                        fields.push(("verdict".to_owned(), verdict.clone()));
                    }
                    GateVerdict::Structured {
                        passed,
                        total,
                        skipped,
                        attacks,
                        msrv,
                    } => {
                        fields.push(("passed".to_owned(), passed.to_string()));
                        fields.push(("total".to_owned(), total.to_string()));
                        if !skipped.is_empty() {
                            fields.push(("skipped".to_owned(), skipped.join(", ")));
                        }
                        fields.push(("attacks".to_owned(), attacks.to_string()));
                        if let Some(msrv) = msrv {
                            fields.push(("msrv".to_owned(), msrv.clone()));
                        }
                    }
                }
            }
            Kind::Landed {
                commit,
                tree,
                evidence,
            } => {
                fields.push(("commit".to_owned(), commit.clone()));
                fields.push(("tree".to_owned(), tree.clone()));
                if *evidence == Evidence::History {
                    fields.push(("evidence".to_owned(), "history".to_owned()));
                }
            }
            Kind::Abandoned { reason } => fields.push(("reason".to_owned(), reason.clone())),
        }
        format::render(&fields)
    }

    /// Путь файла соответствует предмету, времени и виду события. При
    /// совпадении времени допускается числовой суффикс `-N`.
    fn file_matches(&self) -> bool {
        let canonical = relative_path(&self.subject, &self.at, &self.kind, 0);
        if self.file == canonical {
            return true;
        }
        let stem = canonical.strip_suffix(".toml").unwrap_or(&canonical);
        self.file
            .strip_prefix(stem)
            .and_then(|rest| rest.strip_prefix('-'))
            .and_then(|rest| rest.strip_suffix(".toml"))
            .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
    }
}

/// Путь файла события от каталога журнала; `attempt` больше нуля добавляет
/// суффикс для событий с одинаковым временем.
pub fn relative_path(subject: &Subject, at: &str, kind: &Kind, attempt: u32) -> String {
    let compact: String = at.chars().filter(|c| *c != '-' && *c != ':').collect();
    let name = kind.name();
    let id = subject.id();
    if attempt == 0 {
        format!("{id}/{compact}-{name}.toml")
    } else {
        format!("{id}/{compact}-{name}-{attempt}.toml")
    }
}

fn is_hash(text: &str) -> bool {
    matches!(text.len(), 40 | 64)
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format;

    const TREE: &str = "0123456789abcdef0123456789abcdef01234567";

    fn event(file: &str, text: &str) -> Result<Event, String> {
        Event::from_record(file, &format::parse(text).unwrap())
    }

    #[test]
    fn every_kind_round_trips_through_its_file() {
        let at = "2026-09-11T03:15:00Z".to_owned();
        for (subject, kind) in [
            (Subject::Work("w-022".to_owned()), Kind::Started),
            (
                Subject::Work("w-022".to_owned()),
                Kind::Gate {
                    gate: "commit".into(),
                    tree: TREE.into(),
                    verdict: GateVerdict::Prose("GATE OK (11 из 11)".into()),
                },
            ),
            (
                Subject::Work("w-022".to_owned()),
                Kind::Gate {
                    gate: "commit".into(),
                    tree: TREE.into(),
                    verdict: GateVerdict::Structured {
                        passed: 11,
                        total: 12,
                        skipped: vec!["external names".to_owned()],
                        attacks: 20,
                        msrv: Some("1.83.0".to_owned()),
                    },
                },
            ),
            (
                Subject::Work("w-022".to_owned()),
                Kind::Landed {
                    commit: TREE.into(),
                    tree: TREE.into(),
                    evidence: Evidence::History,
                },
            ),
            (
                Subject::Work("w-022".to_owned()),
                Kind::Abandoned {
                    reason: "замещена работой 23".into(),
                },
            ),
            (Subject::Slice("s-007".to_owned()), Kind::Closed),
        ] {
            let written = Event::new(subject, at.clone(), kind);
            let read = event(&written.file, &written.to_text()).unwrap();
            assert_eq!(read, written);
        }
        assert_eq!(
            Event::new(Subject::Work("w-022".to_owned()), at, Kind::Started).file,
            "w-022/20260911T031500Z-started.toml"
        );
    }

    /// Журнал двуформатный (решение 22): прежнее событие с прозой и новое со
    /// структурой читаются одним и тем же разбором.
    #[test]
    fn prose_and_structured_gate_verdicts_are_both_read() {
        let prose = "event = \"gate\"\nwork = \"w-022\"\nat = \"2026-09-11T03:15:00Z\"\ngate = \"commit\"\ntree = \"0123456789abcdef0123456789abcdef01234567\"\nverdict = \"GATE OK (11 из 11)\"\n";
        let read = event("w-022/20260911T031500Z-gate.toml", prose).unwrap();
        assert_eq!(
            read.kind,
            Kind::Gate {
                gate: "commit".into(),
                tree: TREE.into(),
                verdict: GateVerdict::Prose("GATE OK (11 из 11)".into()),
            }
        );

        let structured = "event = \"gate\"\nwork = \"w-022\"\nat = \"2026-09-11T03:15:00Z\"\ngate = \"commit\"\ntree = \"0123456789abcdef0123456789abcdef01234567\"\npassed = \"11\"\ntotal = \"12\"\nskipped = \"external names\"\nattacks = \"20\"\nmsrv = \"1.83.0\"\n";
        let read = event("w-022/20260911T031500Z-gate.toml", structured).unwrap();
        assert_eq!(
            read.kind,
            Kind::Gate {
                gate: "commit".into(),
                tree: TREE.into(),
                verdict: GateVerdict::Structured {
                    passed: 11,
                    total: 12,
                    skipped: vec!["external names".to_owned()],
                    attacks: 20,
                    msrv: Some("1.83.0".to_owned()),
                },
            }
        );
    }

    /// Ярус коммита (решение 33): структурный вердикт без `msrv` читается и
    /// записывается без поля `msrv` — разница ярусов выразима без нового поля.
    #[test]
    fn commit_tier_verdict_omits_msrv_in_the_file() {
        let structured = "event = \"gate\"\nwork = \"w-022\"\nat = \"2026-09-11T03:15:00Z\"\ngate = \"commit\"\ntree = \"0123456789abcdef0123456789abcdef01234567\"\npassed = \"9\"\ntotal = \"9\"\nskipped = \"external names\"\nattacks = \"20\"\n";
        let read = event("w-022/20260911T031500Z-gate.toml", structured).unwrap();
        assert_eq!(
            read.kind,
            Kind::Gate {
                gate: "commit".into(),
                tree: TREE.into(),
                verdict: GateVerdict::Structured {
                    passed: 9,
                    total: 9,
                    skipped: vec!["external names".to_owned()],
                    attacks: 20,
                    msrv: None,
                },
            }
        );
        assert_eq!(read.to_text(), structured);
    }

    #[test]
    fn fields_are_checked_strictly() {
        let file = "w-022/20260911T031500Z-started.toml";
        let base = "work = \"w-022\"\nat = \"2026-09-11T03:15:00Z\"\n";
        for (text, reason) in [
            (base.to_owned(), "no event field"),
            (format!("event = \"begun\"\n{base}"), "unknown event"),
            (
                format!("event = \"started\"\n{base}owner = \"x\"\n"),
                "extra field `owner`",
            ),
            (
                "event = \"started\"\nwork = \"s-022\"\nat = \"2026-09-11T03:15:00Z\"\n".to_owned(),
                "identifier like w-fix-gitignore",
            ),
            (
                "event = \"started\"\nwork = \"w-022\"\nat = \"вчера\"\n".to_owned(),
                "UTC time",
            ),
        ] {
            let error = event(file, &text).expect_err(&text);
            assert!(error.contains(reason), "{text}: {error}");
        }
        let gate = "event = \"gate\"\nwork = \"w-022\"\nat = \"2026-09-11T03:15:00Z\"\ngate = \"commit\"\ntree = \"XYZ\"\nverdict = \"ok\"\n";
        let error = event("w-022/20260911T031500Z-gate.toml", gate).unwrap_err();
        assert!(error.contains("field tree is a git hash"), "{error}");
    }

    #[test]
    fn file_name_must_match_the_event() {
        let text = "event = \"started\"\nwork = \"w-022\"\nat = \"2026-09-11T03:15:00Z\"\n";
        assert!(event("w-022/20260911T031500Z-started-2.toml", text).is_ok());
        let error = event("w0023/20260911T031500Z-started.toml", text).unwrap_err();
        assert!(
            error.contains("expected w-022/20260911T031500Z-started.toml"),
            "{error}"
        );
    }
}
