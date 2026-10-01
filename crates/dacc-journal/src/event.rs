//! События журнала: предмет, время, вид и поля вида.

use crate::format::{self, Record};
use crate::time;

/// Предмет события: единица работы, срез или обязательство, адресуемый slug.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subject {
    Work(String),
    Slice(String),
    /// Обязательство (решение 35). Событий у обязательства нет: погашение —
    /// приземлённая работа, а не событие журнала. Ось нужна для свёртки.
    Obligation(String),
}

impl Subject {
    /// Идентификатор в плане: `w-fix-gitignore`, `s-versioning` или `o-fix-x`.
    pub fn id(&self) -> &str {
        match self {
            Self::Work(slug) | Self::Slice(slug) | Self::Obligation(slug) => slug,
        }
    }

    /// Разбирает `w-fix-gitignore`, `s-versioning` или `o-fix-x`.
    pub fn parse(text: &str) -> Option<Subject> {
        match text.as_bytes() {
            [b'w', b'-', ..] => Some(Self::Work(text.to_owned())),
            [b's', b'-', ..] => Some(Self::Slice(text.to_owned())),
            [b'o', b'-', ..] => Some(Self::Obligation(text.to_owned())),
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

/// Анти-вакуумный вид доказательства готовности (работа w-evidence-kinds).
///
/// Вид несёт предмет — имя теста, проверки или находки. Вид без предмета
/// невыразим: в событии он не записывается и не разбирается, пустое значение
/// отвергается с именем поля.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proof {
    /// Тест или сценарий падал до починки — предмет именует его.
    RedBefore(String),
    /// Проверка ловит внесённые поломки — предмет именует проверку.
    MutationProof(String),
    /// Проверка нашла предмет: «ничего не найдено» не засчитывается успехом.
    AntiVacuum(String),
}

impl Proof {
    /// Ключ поля вида в событии.
    pub fn key(&self) -> &'static str {
        match self {
            Self::RedBefore(_) => "red_before",
            Self::MutationProof(_) => "mutation_proof",
            Self::AntiVacuum(_) => "anti_vacuum",
        }
    }

    /// Предмет вида.
    pub fn subject(&self) -> &str {
        match self {
            Self::RedBefore(subject) | Self::MutationProof(subject) | Self::AntiVacuum(subject) => {
                subject
            }
        }
    }
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
        /// Анти-вакуумные виды доказательства в порядке ключей события.
        proofs: Vec<Proof>,
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
        let (subject, at, kind) = parse_fields(record)?;
        let event = Event {
            file: file.to_owned(),
            subject,
            at,
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

    /// Событие из таблицы одной записи журнала (решение 43): события адресуются
    /// позицией, а не именем файла, поэтому имя файла не сверяется.
    pub fn from_entry(record: &Record, index: usize) -> Result<Event, String> {
        let (subject, at, kind) = parse_fields(record)?;
        Ok(Event {
            file: format!("journal.toml#{index}"),
            subject,
            at,
            kind,
        })
    }

    /// Текст файла события.
    pub fn to_text(&self) -> String {
        let subject_key = match self.subject {
            Subject::Work(_) => "work",
            Subject::Slice(_) => "slice",
            Subject::Obligation(_) => "obligation",
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
                proofs,
            } => {
                fields.push(("commit".to_owned(), commit.clone()));
                fields.push(("tree".to_owned(), tree.clone()));
                if *evidence == Evidence::History {
                    fields.push(("evidence".to_owned(), "history".to_owned()));
                }
                for key in ["red_before", "mutation_proof", "anti_vacuum"] {
                    if let Some(proof) = proofs.iter().find(|proof| proof.key() == key) {
                        fields.push((key.to_owned(), proof.subject().to_owned()));
                    }
                }
            }
            Kind::Abandoned { reason } => fields.push(("reason".to_owned(), reason.clone())),
        }
        format::render(&fields)
    }

    /// Текст события как таблицы `[[events]]` для одной записи журнала
    /// (решение 43): заголовок таблицы и плоские поля события.
    pub fn to_table(&self) -> String {
        format!("[[events]]\n{}", self.to_text())
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

/// Поля события из записи: (subject, at, kind). Имя файла не участвует — его
/// сверяет `from_record`, а `from_entry` событий одной записи не сверяет.
fn parse_fields(record: &Record) -> Result<(Subject, String, Kind), String> {
    let name = record.get("event").ok_or("no event field")?;
    let (subject_key, own): (&str, &[&str]) = match name {
        "started" => ("work", &[]),
        "gate" => (
            "work",
            &[
                "gate", "tree", "verdict", "passed", "total", "skipped", "attacks", "msrv",
            ],
        ),
        "landed" => (
            "work",
            &[
                "commit",
                "tree",
                "evidence",
                "red_before",
                "mutation_proof",
                "anti_vacuum",
            ],
        ),
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
        "landed" => {
            let mut proofs = Vec::new();
            for (key, build) in [
                ("red_before", Proof::RedBefore as fn(String) -> Proof),
                ("mutation_proof", Proof::MutationProof),
                ("anti_vacuum", Proof::AntiVacuum),
            ] {
                // Вид без предмета невыразим: присутствующий ключ обязан нести
                // непустой предмет, иначе отказ с именем поля.
                if record.get(key).is_some() {
                    proofs.push(build(required(key)?));
                }
            }
            Kind::Landed {
                commit: hash("commit")?,
                tree: hash("tree")?,
                evidence: match record.get("evidence") {
                    None => Evidence::Gate,
                    Some("history") => Evidence::History,
                    Some(other) => {
                        return Err(format!("field evidence is only history, not {other:?}"))
                    }
                },
                proofs,
            }
        }
        "abandoned" => Kind::Abandoned {
            reason: required("reason")?,
        },
        _ => Kind::Closed,
    };

    Ok((subject, at.to_owned(), kind))
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
                    proofs: Vec::new(),
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

    /// Решение 43: события одной записи журнала проходят круг через таблицы
    /// `[[events]]` — запись адресуется позицией, а не именем файла.
    #[test]
    fn events_round_trip_through_the_single_journal() {
        let landed = Kind::Landed {
            commit: TREE.into(),
            tree: TREE.into(),
            evidence: Evidence::Gate,
            proofs: Vec::new(),
        };
        let events = [
            Event::new(
                Subject::Work("w-022".to_owned()),
                "2026-09-11T03:15:00Z".to_owned(),
                Kind::Started,
            ),
            Event::new(
                Subject::Work("w-022".to_owned()),
                "2026-09-11T03:15:01Z".to_owned(),
                landed.clone(),
            ),
        ];
        let text = events.iter().map(Event::to_table).collect::<String>();
        let records = crate::format::parse_events(&text).expect("массив таблиц");
        let read: Vec<Event> = records
            .iter()
            .enumerate()
            .map(|(index, record)| Event::from_entry(record, index).expect("событие"))
            .collect();
        assert_eq!(read[0].kind, Kind::Started);
        assert_eq!(read[1].kind, landed);
        assert_eq!(read[0].file, "journal.toml#0");
        assert_eq!(read[1].file, "journal.toml#1");
    }

    /// Анти-вакуумные виды доказательства (работа w-evidence-kinds): каждый вид
    /// несёт предмет, событие landed хранит их плоскими полями и читает прежние
    /// события без правки.
    #[test]
    fn landed_carries_anti_vacuous_proofs() {
        let landed = Kind::Landed {
            commit: TREE.into(),
            tree: TREE.into(),
            evidence: Evidence::Gate,
            proofs: vec![
                Proof::RedBefore("tests::refused_before_the_fix".to_owned()),
                Proof::MutationProof("gate step tests".to_owned()),
                Proof::AntiVacuum("27 attacks found a subject".to_owned()),
            ],
        };
        let written = Event::new(
            Subject::Work("w-022".to_owned()),
            "2026-09-11T03:15:00Z".to_owned(),
            landed.clone(),
        );
        let text = written.to_text();
        assert!(
            text.contains("red_before = \"tests::refused_before_the_fix\""),
            "{text}"
        );
        assert!(
            text.contains("mutation_proof = \"gate step tests\""),
            "{text}"
        );
        assert!(
            text.contains("anti_vacuum = \"27 attacks found a subject\""),
            "{text}"
        );
        let read = event(&written.file, &text).unwrap();
        assert_eq!(read, written);
    }

    /// Вид доказательства без предмета невыразим: пустое значение отвергается
    /// с именем поля, а не превращается молча в отсутствие вида.
    #[test]
    fn proof_without_a_subject_is_refused() {
        let text = "event = \"landed\"\nwork = \"w-022\"\nat = \"2026-09-11T03:15:00Z\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\ntree = \"0123456789abcdef0123456789abcdef01234567\"\nred_before = \"\"\n";
        let error = event("w-022/20260911T031500Z-landed.toml", text).unwrap_err();
        assert!(error.contains("red_before"), "{error}");
    }

    /// Прежние события без видов доказательства читаются без правки: поля
    /// доказательства отсутствуют, и это не ошибка.
    #[test]
    fn previous_landed_without_proofs_reads_unchanged() {
        let text = "event = \"landed\"\nwork = \"w-022\"\nat = \"2026-09-11T03:15:00Z\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\ntree = \"0123456789abcdef0123456789abcdef01234567\"\nevidence = \"history\"\n";
        let read = event("w-022/20260911T031500Z-landed.toml", text).unwrap();
        assert_eq!(
            read.kind,
            Kind::Landed {
                commit: TREE.into(),
                tree: TREE.into(),
                evidence: Evidence::History,
                proofs: Vec::new(),
            }
        );
        assert_eq!(read.to_text(), text);
    }
}
