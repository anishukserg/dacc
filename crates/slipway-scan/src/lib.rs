//! Скан реестра решений и порождение модуля констант.
//!
//! Ключевой механизм методологии (решения 1 и 4):
//! ссылка между документами — путь к константе, а не число. Константы
//! порождаются здесь, из текста реестра, поэтому удаление документа делает
//! ссылку на него не резолвящейся — обычной ошибкой компилятора.
//!
//! Работает с ТЕКСТОМ файлов, а не со скомпилированным крейтом: цикла
//! «реестр порождает константы, которыми пользуется сам» не возникает.
//! Цена текстового разбора — второй источник истины: скан классифицирует
//! запись по токенам, компилятор вычисляет её значение. Поэтому порождённый
//! код сверяет одно с другим константными утверждениями, и расхождение
//! становится ошибкой компиляции, а не тихо неверным модулем констант.

// Нестабильные возможности запрещены и в doctest: lints манифеста на них не
// распространяются, а атаки выполняются под RUSTC_BOOTSTRAP (решение 12).
#![doc(test(attr(forbid(unstable_features))))]

pub mod anchors;
pub mod journal;
pub mod plan;
pub use anchors::{emit_anchors, scan_anchors, AnchorMode, ScannedAnchor};

use std::{fmt::Write as _, fs, path::Path};

use slipway_core::anchor_rules::slug_ident;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedDecision {
    pub id: u32,
    pub module: String,
    pub status: Status,
    /// Абсолютный путь к файлу. Нужен потому, что порождённый модуль лежит
    /// в OUT_DIR, а `#[path]` разрешается относительно него.
    pub file: String,
}

/// Запись скана слоя знания: идентификатор — slug из имени файла.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedSlug {
    /// Идентификатор документа — имя файла без расширения (`adr-2026-001`).
    pub slug: String,
    pub status: Status,
    /// Абсолютный путь к файлу.
    pub file: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Draft,
    Active,
    Deprecated,
    SupersededBy,
}

/// Конструкторы, допустимые только в порождённом коде. Их появление в тексте
/// реестра — обход невыразимости, а не опечатка.
const BYPASS_IDENTS: [&str; 2] = ["__from_scan", "__new_unchecked"];

#[derive(Debug)]
pub enum ScanError {
    Io(String),
    Parse {
        file: String,
        detail: String,
    },
    /// Имя файла не каноническое для идентификатора внутри него.
    IdMismatch {
        file: String,
        declared: u32,
        expected: String,
    },
    /// В тексте реестра вызван конструктор, предназначенный скану.
    Bypass {
        file: String,
        ident: String,
    },
    /// Разметка кода записана с ошибкой.
    Anchor {
        file: String,
        line: usize,
        detail: String,
    },
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(m) => write!(f, "i/o: {m}"),
            Self::Parse { file, detail } => write!(f, "{file}: not parsed: {detail}"),
            Self::IdMismatch { file, declared, expected } => {
                write!(f, "{file}: identifier {declared} requires the file name {expected}.rs")
            }
            Self::Bypass { file, ident } => write!(
                f,
                "{file}: `{ident}` is allowed in generated code only; a reference in the registry is a path to a constant"
            ),
            Self::Anchor { file, line, detail } => write!(f, "{file}:{line}: code anchor: {detail}"),
        }
    }
}

/// Сканирует каталог реестра спецификаций (`rfc!`).
pub fn scan_specs(dir: &Path) -> Result<Vec<ScannedSlug>, ScanError> {
    scan_slug_dir(dir, "rfc")
}

/// Сканирует каталог реестра решений (`adr!`).
pub fn scan_decisions(dir: &Path) -> Result<Vec<ScannedSlug>, ScanError> {
    scan_slug_dir(dir, "adr")
}

fn scan_dir(dir: &Path, macro_name: &str, prefix: char) -> Result<Vec<ScannedDecision>, ScanError> {
    let mut found = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|e| ScanError::Io(format!("{}: {e}", dir.display())))?;

    for entry in entries {
        let path = entry.map_err(|e| ScanError::Io(e.to_string()))?.path();
        let is_rs = path.extension().is_some_and(|e| e == "rs");
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        if !is_rs || stem == "mod" {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|e| ScanError::Io(e.to_string()))?;
        let mut decision = parse_entry(&text, &stem, macro_name, prefix)?;
        decision.file = path.display().to_string();
        found.push(decision);
    }
    found.sort_by_key(|d| d.id);
    Ok(found)
}

/// Сканирует каталог реестра слоя знания: идентификатор каждого документа —
/// имя его файла (slug), а не число внутри макроса.
fn scan_slug_dir(dir: &Path, macro_name: &str) -> Result<Vec<ScannedSlug>, ScanError> {
    let mut found = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|e| ScanError::Io(format!("{}: {e}", dir.display())))?;

    for entry in entries {
        let path = entry.map_err(|e| ScanError::Io(e.to_string()))?.path();
        let is_rs = path.extension().is_some_and(|e| e == "rs");
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        if !is_rs || stem == "mod" {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|e| ScanError::Io(e.to_string()))?;
        let mut decision = parse_slug_entry(&text, &stem, macro_name)?;
        decision.file = path.display().to_string();
        found.push(decision);
    }
    found.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(found)
}

/// Разбирает один файл реестра слоя знания: находит вызов макроса
/// регистрации, извлекает статус, а идентификатор принимает из имени файла.
pub fn parse_slug_entry(
    text: &str,
    module: &str,
    macro_name: &str,
) -> Result<ScannedSlug, ScanError> {
    let file = syn::parse_file(text).map_err(|e| ScanError::Parse {
        file: module.into(),
        detail: e.to_string(),
    })?;

    let mac = file
        .items
        .iter()
        .find_map(|item| match item {
            // Путь может быть как `adr!`, так и `slipway_knowledge::adr!` —
            // значим только последний сегмент.
            syn::Item::Macro(m)
                if m.mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == macro_name) =>
            {
                Some(&m.mac)
            }
            _ => None,
        })
        .ok_or_else(|| ScanError::Parse {
            file: module.into(),
            detail: format!("no {macro_name}! call found"),
        })?;

    // Идентификатор — само имя файла, и оно обязано быть допустимым slug.
    // Инъективность отображения «slug → имя константы» гарантирует, что два
    // документа не дадут одну константу (атака E2 в слое знания).
    slipway_core::anchor_rules::check_slug_id(module).map_err(|detail| ScanError::Parse {
        file: module.into(),
        detail,
    })?;

    if let Some(ident) = find_bypass(text) {
        return Err(ScanError::Bypass {
            file: module.into(),
            ident,
        });
    }

    let tokens: Vec<_> = mac.tokens.clone().into_iter().collect();
    let status = extract_status(&tokens);
    Ok(ScannedSlug {
        slug: module.to_owned(),
        status,
        file: String::new(),
    })
}

/// Разбирает один файл реестра решений: идентификатор — имя файла (slug).
pub fn parse_decision(text: &str, module: &str) -> Result<ScannedSlug, ScanError> {
    parse_slug_entry(text, module, "adr")
}

/// Разбирает один файл реестра: находит вызов макроса регистрации и
/// извлекает идентификатор и статус из его токенов.
pub fn parse_entry(
    text: &str,
    module: &str,
    macro_name: &str,
    prefix: char,
) -> Result<ScannedDecision, ScanError> {
    let file = syn::parse_file(text).map_err(|e| ScanError::Parse {
        file: module.into(),
        detail: e.to_string(),
    })?;

    let mac = file
        .items
        .iter()
        .find_map(|item| match item {
            // Путь может быть как `adr!`, так и `slipway_knowledge::adr!` —
            // значим только последний сегмент.
            syn::Item::Macro(m)
                if m.mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == macro_name) =>
            {
                Some(&m.mac)
            }
            _ => None,
        })
        .ok_or_else(|| ScanError::Parse {
            file: module.into(),
            detail: format!("no {macro_name}! call found"),
        })?;

    let tokens: Vec<_> = mac.tokens.clone().into_iter().collect();
    let id = tokens
        .iter()
        .find_map(|t| match t {
            proc_macro2::TokenTree::Literal(l) => l.to_string().parse::<u32>().ok(),
            _ => None,
        })
        .ok_or_else(|| ScanError::Parse {
            file: module.into(),
            detail: "no identifier found".into(),
        })?;

    // Каноническое имя файла: у идентификатора ровно одно допустимое имя,
    // поэтому второй файл с тем же идентификатором в каталоге невыразим —
    // ни через ведущие нули, ни через имя без номера.
    let expected = format!("{prefix}{id:04}");
    if module != expected {
        return Err(ScanError::IdMismatch {
            file: module.into(),
            declared: id,
            expected,
        });
    }

    if let Some(ident) = find_bypass(text) {
        return Err(ScanError::Bypass {
            file: module.into(),
            ident,
        });
    }

    let status = extract_status(&tokens);
    Ok(ScannedDecision {
        id,
        module: module.to_owned(),
        status,
        file: String::new(),
    })
}

/// Ищет конструкторы скана во всём тексте файла, включая вложенные группы.
/// Проза в строковых литералах и комментарии токенами-идентификаторами не
/// являются, поэтому упоминание в тексте решения ложной тревоги не даёт.
fn find_bypass(text: &str) -> Option<String> {
    let stream: proc_macro2::TokenStream = text.parse().ok()?;
    find_bypass_in(stream)
}

fn find_bypass_in(stream: proc_macro2::TokenStream) -> Option<String> {
    stream.into_iter().find_map(|t| match t {
        proc_macro2::TokenTree::Ident(i) => {
            let name = i.to_string();
            BYPASS_IDENTS.contains(&name.as_str()).then_some(name)
        }
        proc_macro2::TokenTree::Group(g) => find_bypass_in(g.stream()),
        _ => None,
    })
}

/// Классификация по токенам верхнего уровня. Может ошибиться на необычной
/// записи статуса — поэтому порождённый код сверяет её с вычисленным
/// значением (`emit_refs`).
fn extract_status(tokens: &[proc_macro2::TokenTree]) -> Status {
    let idents: Vec<String> = tokens
        .iter()
        .filter_map(|t| match t {
            proc_macro2::TokenTree::Ident(i) => Some(i.to_string()),
            _ => None,
        })
        .collect();

    let after_status = idents
        .iter()
        .position(|i| i == "status")
        .and_then(|p| idents.get(p + 1..));

    match after_status.and_then(|rest| {
        rest.iter().find(|i| {
            matches!(
                i.as_str(),
                "Draft" | "Active" | "Deprecated" | "SupersededBy"
            )
        })
    }) {
        Some(s) if s == "Active" => Status::Active,
        Some(s) if s == "Deprecated" => Status::Deprecated,
        Some(s) if s == "SupersededBy" => Status::SupersededBy,
        _ => Status::Draft,
    }
}

/// Порождает модуль констант и агрегированный список.
///
/// Константы разложены по подмодулям так, чтобы ссылка не того класса была
/// невыразима: `Retirement` принимает только `superseded::*`.
pub fn emit_refs(decisions: &[ScannedSlug]) -> String {
    let mut out = String::from("// GENERATED by slipway-scan. Do not edit.\n\n");

    // Каждый публичный элемент документирован: порождённый код собирается под
    // строгим профилем lints продукта (решение 16).
    for d in decisions {
        let ident = slug_ident(&d.slug);
        let slug = d.slug.as_str();
        let file = d.file.as_str();
        let _ = writeln!(
            out,
            "/// Decision {slug}.\n#[path = {file:?}]\npub mod {ident};"
        );
    }

    out.push_str("\n/// References to decisions: a path to a constant instead of a number.\n#[allow(non_upper_case_globals, unused_imports)]\npub mod adr {\n    use slipway_core::AdrRef;\n");
    for d in decisions {
        let ident = slug_ident(&d.slug);
        let slug = d.slug.as_str();
        let _ = writeln!(
            out,
            "    /// Reference to decision {slug}.\n    pub const {ident}: AdrRef = AdrRef::__from_scan({slug:?});"
        );
    }
    out.push_str("}\n\n");

    out.push_str("/// Superseded decisions only: retiring live code is inexpressible.\n");
    out.push_str("#[allow(non_upper_case_globals, unused_imports)]\npub mod superseded {\n    use slipway_core::SupersededRef;\n");
    for d in decisions
        .iter()
        .filter(|d| d.status == Status::SupersededBy)
    {
        let ident = slug_ident(&d.slug);
        let slug = d.slug.as_str();
        let _ = writeln!(
            out,
            "    /// Reference to superseded decision {slug}.\n    pub const {ident}: SupersededRef = SupersededRef::__from_scan({slug:?});"
        );
    }
    out.push_str("}\n\n");

    out.push_str(
        "// Scan cross-checked against the compiler: the scan classifies by tokens, the\n",
    );
    out.push_str(
        "// compiler evaluates the value. A divergence is a constant evaluation error (E0080).\n",
    );
    for d in decisions {
        let ident = slug_ident(&d.slug);
        let slug = d.slug.as_str();
        let (neg, what) = if d.status == Status::SupersededBy {
            ("", "superseded")
        } else {
            ("!", "not superseded")
        };
        let _ = writeln!(
            out,
            "const _: () = assert!({neg}matches!({ident}::DECISION.status, ::slipway_knowledge::DocStatus::SupersededBy(_)), \"slipway-scan: by its text {slug} is {what}, the compiler evaluated otherwise\");"
        );
    }

    out.push_str(
        "\n/// All decisions of the registry: the slug id and the document.\npub static ALL: &[(&str, &slipway_knowledge::ArchitectureDecision)] = &[\n",
    );
    for d in decisions {
        let ident = slug_ident(&d.slug);
        let slug = d.slug.as_str();
        let _ = writeln!(out, "    ({slug:?}, &{ident}::DECISION),");
    }
    out.push_str("];\n");
    out
}

/// Порождает модуль констант для реестра спецификаций.
pub fn emit_spec_refs(specs: &[ScannedSlug]) -> String {
    let mut out = String::from("// GENERATED by slipway-scan. Do not edit.\n\n");
    for s in specs {
        let ident = slug_ident(&s.slug);
        let slug = s.slug.as_str();
        let file = s.file.as_str();
        let _ = writeln!(
            out,
            "/// Specification {slug}.\n#[path = {file:?}]\npub mod {ident};"
        );
    }
    out.push_str("\n/// References to specifications: a path to a constant instead of a number.\n#[allow(non_upper_case_globals, unused_imports)]\npub mod rfc {\n    use slipway_core::RfcRef;\n");
    for s in specs {
        let ident = slug_ident(&s.slug);
        let slug = s.slug.as_str();
        let _ = writeln!(
            out,
            "    /// Reference to specification {slug}.\n    pub const {ident}: RfcRef = RfcRef::__from_scan({slug:?});"
        );
    }
    out.push_str("}\n\n");
    out.push_str(
        "\n/// All specifications of the registry: the slug id and the document.\npub static ALL_SPECS: &[(&str, &slipway_knowledge::DomainSpecification)] = &[\n",
    );
    for s in specs {
        let ident = slug_ident(&s.slug);
        let slug = s.slug.as_str();
        let _ = writeln!(out, "    ({slug:?}, &{ident}::SPEC),");
    }
    out.push_str("];\n");
    out
}

/// Библиотечная точка входа раскладки реестра (решение 27): скан решений,
/// спецификаций, плана и журнала, порождение и запись всех файлов в `out_dir`.
/// build.rs потребителя сводится к одному вызову; каталоги реестра ожидаются в
/// корне `registry_dir` — он же корень крейта документов.
pub fn emit_registry(registry_dir: &Path, out_dir: &Path) -> Result<(), ScanError> {
    let decisions = scan_decisions(&registry_dir.join("adr"))?;
    write_file(out_dir, "adr.rs", &emit_refs(&decisions))?;

    let specs = scan_specs(&registry_dir.join("rfc"))?;
    write_file(out_dir, "rfc.rs", &emit_spec_refs(&specs))?;

    let thrusts = plan::scan_plan(&registry_dir.join("thrust"), &plan::THRUSTS)?;
    let slices = plan::scan_plan(&registry_dir.join("slice"), &plan::SLICES)?;
    let work = plan::scan_plan(&registry_dir.join("work"), &plan::WORK)?;
    let mut code = plan::emit_plan(&thrusts, &plan::THRUSTS);
    code.push_str(&plan::emit_plan(&slices, &plan::SLICES));
    code.push_str(&plan::emit_plan(&work, &plan::WORK));
    code.push_str(&plan::emit_work_checks(&work));
    write_file(out_dir, "plan.rs", &code)?;

    let journal = journal::scan_journal(&registry_dir.join("journal"), &work)?;
    write_file(out_dir, "journal.rs", &journal)?;

    write_file(out_dir, "commit.rs", &commit_constant())?;

    for dir in ["adr", "rfc", "thrust", "slice", "work", "journal"] {
        println!("cargo::rerun-if-changed={dir}");
    }
    Ok(())
}

fn write_file(out_dir: &Path, name: &str, text: &str) -> Result<(), ScanError> {
    fs::write(out_dir.join(name), text).map_err(|error| ScanError::Io(format!("{name}: {error}")))
}

/// Коммит, из которого собран реестр; вне репозитория git — неизвестен.
fn commit_constant() -> String {
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default();
    format!(
        "/// The git commit this registry was built from.\npub static COMMIT: &str = {commit:?};\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        use slipway_knowledge::DocStatus;
        slipway_knowledge::adr!(
            title: "Прямая передача плана",
            status: DocStatus::Active,
        );
    "#;

    #[test]
    fn extracts_status_and_slug() {
        let d = parse_decision(SAMPLE, "adr-direct-plan").unwrap();
        assert_eq!(d.slug, "adr-direct-plan");
        assert_eq!(d.status, Status::Active);
    }

    #[test]
    fn rejects_non_slug_file_name() {
        let err = parse_decision(SAMPLE, "ADR-2026-001").unwrap_err();
        assert!(err.to_string().contains("slug id"), "{err}");
    }

    #[test]
    fn superseded_module_holds_only_superseded() {
        let ds = vec![
            ScannedSlug {
                slug: "adr-text-sql-path".into(),
                status: Status::SupersededBy,
                file: "/x/adr-text-sql-path.rs".into(),
            },
            ScannedSlug {
                slug: "adr-direct-plan".into(),
                status: Status::Active,
                file: "/x/adr-direct-plan.rs".into(),
            },
        ];
        let out = emit_refs(&ds);
        assert!(out.contains("pub const adr_text_sql_path: SupersededRef"));
        assert!(!out.contains("pub const adr_direct_plan: SupersededRef"));
        assert!(out.contains("pub const adr_direct_plan: AdrRef"));
    }

    #[test]
    fn emitted_code_cross_checks_scan_against_compiler() {
        let ds = vec![
            ScannedSlug {
                slug: "adr-text-sql-path".into(),
                status: Status::SupersededBy,
                file: "/x/adr-text-sql-path.rs".into(),
            },
            ScannedSlug {
                slug: "adr-direct-plan".into(),
                status: Status::Active,
                file: "/x/adr-direct-plan.rs".into(),
            },
        ];
        let out = emit_refs(&ds);
        assert!(out.contains("assert!(matches!(adr_text_sql_path::DECISION.status"));
        assert!(out.contains("assert!(!matches!(adr_direct_plan::DECISION.status"));
    }

    /// Каталог реестра во временной папке: имя файла → содержимое.
    fn registry(tag: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("slipway-scan-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for (name, text) in files {
            fs::write(dir.join(name), text).unwrap();
        }
        dir
    }

    const ACTIVE: &str = "slipway_knowledge::adr!(status: DocStatus::Active,);";

    /// Позитивный контроль для атак ниже: корректный каталог принимается.
    #[test]
    fn accepts_slug_registry() {
        let dir = registry(
            "ok",
            &[
                ("adr-direct-plan.rs", ACTIVE),
                ("adr-text-sql-path.rs", ACTIVE),
            ],
        );
        assert_eq!(scan_decisions(&dir).unwrap().len(), 2);
    }

    /// Атака E2 в слое знания: подчёркивание в slug запрещено, иначе два
    /// имени (`adr-text-sql` и `adr_text_sql`) дали бы одну константу.
    #[test]
    fn rejects_underscore_in_slug() {
        let dir = registry("underscore", &[("adr_text_sql_path.rs", ACTIVE)]);
        let err = scan_decisions(&dir).expect_err("slug с подчёркиванием принят");
        assert!(err.to_string().contains("slug id"), "{err}");
    }

    /// Имя файла с заглавными буквами не является slug.
    #[test]
    fn rejects_uppercase_file_name() {
        let dir = registry("upper", &[("ADR-2026-001.rs", ACTIVE)]);
        let err = scan_decisions(&dir).expect_err("файл с заглавными принят");
        assert!(err.to_string().contains("slug id"), "{err}");
    }

    /// Атака E1 в обход закрытого поля: конструктор скана, вызванный в реестре руками.
    #[test]
    fn rejects_scan_constructor_in_registry_text() {
        let text = "slipway_knowledge::adr!(status: DocStatus::SupersededBy(slipway_core::AdrRef::__from_scan(\"adr-2026-002\")),);";
        let err = parse_decision(text, "adr-2026-001").expect_err("ручной __from_scan принят");
        assert!(err.to_string().contains("__from_scan"), "{err}");
    }

    /// Снимок порождаемого текста: весь он английский (решение 22).
    ///
    /// Порождённый код попадает на страницы `cargo doc` чужого реестра, и
    /// кириллица там — смешение языков у стороннего потребителя. Контроль
    /// краснеет на возврат русского текста в любую из порождаемых форм:
    /// шапку, документацию, сообщения константных проверок.
    #[test]
    fn generated_text_carries_no_cyrillic() {
        let ds = vec![
            ScannedSlug {
                slug: "adr-text-sql-path".into(),
                status: Status::SupersededBy,
                file: "/x/adr-text-sql-path.rs".into(),
            },
            ScannedSlug {
                slug: "adr-direct-plan".into(),
                status: Status::Active,
                file: "/x/adr-direct-plan.rs".into(),
            },
        ];
        let work = vec![ScannedDecision {
            id: 1,
            module: "w0001".into(),
            status: Status::Draft,
            file: "/x/w0001.rs".into(),
        }];
        // Проверка до начала работы — нарушение автомата, закрытый срез при
        // незавершённой работе — константная проверка: оба пути порождают
        // сообщение, половина которого приходит из slipway-journal.
        let events = [
            slipway_journal::Event::new(
                slipway_journal::Subject::Work(1),
                "2026-09-11T03:15:01Z".to_owned(),
                slipway_journal::Kind::Gate {
                    gate: "commit".into(),
                    tree: "a".repeat(40),
                    verdict: "ok".into(),
                },
            ),
            slipway_journal::Event::new(
                slipway_journal::Subject::Slice(3),
                "2026-09-11T03:15:02Z".to_owned(),
                slipway_journal::Kind::Closed,
            ),
        ];
        // Нечитаемый файл события: причина приходит из разбора формата.
        let unreadable = slipway_journal::parse_event("w0001/x.toml", "[table]\n").unwrap_err();
        let found =
            anchors::anchors_in_file("#[doc_anchor(id = \"plan-ir\")]\npub struct P;\n", "p.rs")
                .unwrap();

        let journal_code = journal::emit_journal(&events, &[unreadable], &work);
        assert!(
            journal_code.contains("compile_error!"),
            "снимок перестал покрывать нарушения журнала: {journal_code}"
        );

        let generated = [
            emit_refs(&ds),
            emit_spec_refs(&ds),
            plan::emit_plan(&work, &plan::THRUSTS),
            plan::emit_plan(&work, &plan::SLICES),
            plan::emit_plan(&work, &plan::WORK),
            plan::emit_work_checks(&work),
            journal_code,
            anchors::emit_anchor_refs(&found),
        ];
        for text in &generated {
            let cyrillic = text.chars().find(|c| matches!(c, '\u{0400}'..='\u{04FF}'));
            assert!(cyrillic.is_none(), "кириллица в порождённом коде: {text}");
        }
    }

    /// Контроль к атаке выше: упоминание конструктора в прозе решения —
    /// строковый литерал, а не вызов, и отвергаться не должно.
    #[test]
    fn mention_in_prose_is_not_a_bypass() {
        let text = "slipway_knowledge::adr!(status: DocStatus::Active, context: r#\"вызов __from_scan руками запрещён\"#,);";
        assert!(parse_decision(text, "adr-2026-001").is_ok());
    }

    /// Точка входа раскладки (решение 27): пишет все порождённые файлы реестра.
    #[test]
    fn emit_registry_writes_all_generated_files() {
        let root = std::env::temp_dir().join(format!("slipway-registry-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for dir in ["adr", "rfc", "thrust", "slice", "work", "journal"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        fs::write(root.join("adr/adr-direct-plan.rs"), ACTIVE).unwrap();
        let out = root.join("out");
        fs::create_dir_all(&out).unwrap();

        emit_registry(&root, &out).unwrap();

        for name in ["adr.rs", "rfc.rs", "plan.rs", "journal.rs", "commit.rs"] {
            assert!(out.join(name).is_file(), "{name} is missing");
        }
        let adr = fs::read_to_string(out.join("adr.rs")).unwrap();
        assert!(adr.contains("adr_direct_plan"), "{adr}");
    }
}
