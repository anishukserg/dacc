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
pub use anchors::{emit_anchor_refs, emit_anchors, scan_anchors, AnchorMode, ScannedAnchor};

use std::{collections::HashSet, fmt::Write as _, fs, path::Path};

use dacc_core::anchor_rules::slug_ident;

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
    /// Инлайн-ссылка прозы указывает на несуществующую разметку (решение 24).
    InlineLink {
        file: String,
        detail: String,
    },
    /// Скан-паттерн протух: положительный контроль перестал матчиться, и ноль
    /// сырых совпадений выглядел бы как чистое дерево, а не как умершая
    /// проверка (anti-vacuity).
    DeadPattern {
        ident: String,
    },
}

impl ScanError {
    /// Способ исправления — обязательная подсказка «как чинить». У каждого
    /// варианта она есть по построению, поэтому нарушение без способа
    /// исправления невыразимо.
    pub const fn fix(&self) -> &'static str {
        match self {
            Self::Io(_) => "check the path and permissions of the file",
            Self::Parse { .. } => "correct the syntax of the file",
            Self::IdMismatch { .. } => "rename the file to match the identifier",
            Self::Bypass { .. } => "reference the constant instead of calling the constructor",
            Self::Anchor { .. } => "correct the anchor id, mode or key",
            Self::InlineLink { .. } => "mark the referenced symbol with #[doc_anchor]",
            Self::DeadPattern { .. } => {
                "restore the generated constructor or update the scan pattern"
            }
        }
    }
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(m) => write!(f, "i/o: {m} — fix: {}", self.fix()),
            Self::Parse { file, detail } => write!(f, "{file}: not parsed: {detail} — fix: {}", self.fix()),
            Self::IdMismatch { file, declared, expected } => write!(
                f,
                "{file}: identifier {declared} requires the file name {expected}.rs — fix: {}",
                self.fix()
            ),
            Self::Bypass { file, ident } => write!(
                f,
                "{file}: `{ident}` is allowed in generated code only; a reference in the registry is a path to a constant — fix: {}",
                self.fix()
            ),
            Self::Anchor { file, line, detail } => {
                write!(f, "{file}:{line}: code anchor: {detail} — fix: {}", self.fix())
            }
            Self::InlineLink { file, detail } => {
                write!(f, "{file}: inline link: {detail} — fix: {}", self.fix())
            }
            Self::DeadPattern { ident } => write!(
                f,
                "anti-vacuity: the scan pattern for `{ident}` is dead — its positive control no longer matches — fix: {}",
                self.fix()
            ),
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

/// Извлекает из прозы ссылки формы `` `[id]` `` (решение 24): обратные кавычки,
/// квадратные скобки с slug-идентификатором разметки, закрывающие обратные
/// кавычки. Только эта форма проверяется; свободное упоминание имени символа
/// ссылкой не является и не извлекается.
pub fn extract_inline_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let Some(pos) = text[i..].find('`') else {
            break;
        };
        let at = i + pos;
        let after = &text[at + 1..];
        if let Some(rest) = after.strip_prefix('[') {
            if let Some(end) = rest.find(']') {
                let id = &rest[..end];
                if rest[end + 1..].starts_with('`')
                    && dacc_core::anchor_rules::check_slug_id(id).is_ok()
                {
                    links.push(id.to_owned());
                }
            }
        }
        i = at + 1;
    }
    links
}

/// Сверяет инлайн-ссылки прозы файла реестра с разметкой: несуществующий
/// якорь — отказ (решение 24).
pub fn check_inline_links(
    text: &str,
    file: &str,
    anchors: &HashSet<&str>,
) -> Result<(), ScanError> {
    for id in extract_inline_links(text) {
        if !anchors.contains(id.as_str()) {
            return Err(ScanError::InlineLink {
                file: file.to_owned(),
                detail: format!(
                    "prose link `[{id}]` references an anchor that is not marked with `#[doc_anchor]`"
                ),
            });
        }
    }
    Ok(())
}

/// Сверяет инлайн-ссылки всех файлов каталога реестра (`adr` или `rfc`).
fn check_inline_links_in_dir(dir: &Path, anchors: &HashSet<&str>) -> Result<(), ScanError> {
    let entries =
        fs::read_dir(dir).map_err(|e| ScanError::Io(format!("{}: {e}", dir.display())))?;
    for entry in entries {
        let path = entry.map_err(|e| ScanError::Io(e.to_string()))?.path();
        let is_rs = path.extension().is_some_and(|e| e == "rs");
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if !is_rs || stem == "mod" {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|e| ScanError::Io(e.to_string()))?;
        check_inline_links(&text, &path.display().to_string(), anchors)?;
    }
    Ok(())
}

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
            // Путь может быть как `adr!`, так и `dacc_knowledge::adr!` —
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
    dacc_core::anchor_rules::check_slug_id(module).map_err(|detail| ScanError::Parse {
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

/// Anti-vacuity: скан-паттерн, у которого ноль сырых совпадений, неотличим от
/// чистого дерева. Положительный контроль держит паттерн живым: конструктор,
/// который скан ищет как обход, обязан по-прежнему порождаться самим сканом.
/// Если порождённый код перестал нести это имя, поиск протух — ошибка сборки,
/// а не тихо чистое дерево.
fn check_bypass_patterns_live(generated: &str) -> Result<(), ScanError> {
    // Скан сам порождает `__from_scan` в каждом модуле ссылок; `__new_unchecked`
    // порождается макросами объявления dacc-core и жив, пока те компилируются.
    // Скан держит положительный контроль за тем именем, которое сам наблюдает.
    if !generated.contains("__from_scan") {
        return Err(ScanError::DeadPattern {
            ident: "__from_scan".to_owned(),
        });
    }
    Ok(())
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
    let mut out = String::from("// GENERATED by dacc-scan. Do not edit.\n\n");

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

    out.push_str("\n/// References to decisions: a path to a constant instead of a number.\n#[allow(non_upper_case_globals, unused_imports)]\npub mod adr {\n    use dacc_core::AdrRef;\n");
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
    out.push_str("#[allow(non_upper_case_globals, unused_imports)]\npub mod superseded {\n    use dacc_core::SupersededRef;\n");
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
            "const _: () = assert!({neg}matches!({ident}::DECISION.status, ::dacc_knowledge::DocStatus::SupersededBy(_)), \"dacc-scan: by its text {slug} is {what}, the compiler evaluated otherwise\");"
        );
    }

    out.push_str(
        "\n/// All decisions of the registry: the slug id and the document.\npub static ALL: &[(&str, &dacc_knowledge::ArchitectureDecision)] = &[\n",
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
    let mut out = String::from("// GENERATED by dacc-scan. Do not edit.\n\n");
    for s in specs {
        let ident = slug_ident(&s.slug);
        let slug = s.slug.as_str();
        let file = s.file.as_str();
        let _ = writeln!(
            out,
            "/// Specification {slug}.\n#[path = {file:?}]\npub mod {ident};"
        );
    }
    out.push_str("\n/// References to specifications: a path to a constant instead of a number.\n#[allow(non_upper_case_globals, unused_imports)]\npub mod rfc {\n    use dacc_core::RfcRef;\n");
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
        "\n/// All specifications of the registry: the slug id and the document.\npub static ALL_SPECS: &[(&str, &dacc_knowledge::DomainSpecification)] = &[\n",
    );
    for s in specs {
        let ident = slug_ident(&s.slug);
        let slug = s.slug.as_str();
        let _ = writeln!(out, "    ({slug:?}, &{ident}::SPEC),");
    }
    out.push_str("];\n");
    out
}

/// Порядковый номер из slug вида `adr-2026-047`: серия `adr-2026`, номер 47.
/// Slug без числового хвоста после года номера не имеет и серии не начинает.
fn ordinal_of(slug: &str) -> Option<(String, u16)> {
    let (series, ordinal) = slug.rsplit_once('-')?;
    if ordinal.is_empty() || !ordinal.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let year = series.rsplit_once('-').map_or(series, |(_, tail)| tail);
    if year.len() != 4 || !year.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    ordinal
        .parse()
        .ok()
        .map(|number| (series.to_owned(), number))
}

/// Имя класса статуса — вариант порождённого перечисления и текст отказа.
fn class_name(status: Status) -> &'static str {
    match status {
        Status::Draft => "Draft",
        Status::Active => "Active",
        Status::Deprecated => "Deprecated",
        Status::SupersededBy => "SupersededBy",
    }
}

/// Счётчики одного вида документов в модуле `counts`.
fn emit_counts(out: &mut String, label: &str, singular: &str, entries: &[ScannedSlug]) {
    let count = |class: Status| entries.iter().filter(|e| e.status == class).count();
    let _ = writeln!(
        out,
        "    /// {singular} in the registry.\n    pub const {label}: usize = {};",
        entries.len()
    );
    for (class, suffix) in [
        (Status::Draft, "DRAFT"),
        (Status::Active, "ACTIVE"),
        (Status::Deprecated, "DEPRECATED"),
        (Status::SupersededBy, "SUPERSEDED"),
    ] {
        let name = class_name(class).to_ascii_lowercase();
        let _ = writeln!(
            out,
            "    /// {singular} classified as {name} by the scan.\n    pub const {label}_{suffix}: usize = {};",
            count(class)
        );
    }
}

/// Порождает модуль выводимых значений (работа w-derived-prose): порядковые
/// номера, статусы и счётчики считает скан, а не человек — git log и grep для
/// них больше не нужны. Граница выводимого и смысловой прозы названа в самих
/// константах: проза остаётся предметом ревью и сюда не попадает.
pub fn emit_derived(decisions: &[ScannedSlug], specs: &[ScannedSlug]) -> String {
    let mut out = String::from("// GENERATED by dacc-scan. Do not edit.\n");
    out.push_str("//\n");
    out.push_str("// Derived and prose: the boundary is named here (work w-derived-prose).\n");
    out.push_str("// The scan computes every value below; a person neither writes nor recounts\n");
    out.push_str("// them. The semantic prose of the records -- context, decision, trade-offs,\n");
    out.push_str("// constraints, goals, invariants -- stays under human review and is never\n");
    out.push_str("// derived from the text.\n\n");

    out.push_str(
        "/// The boundary of the derived: these values the scan computes.\npub const DERIVED_BY_SCAN: &[&str] = &[\"ordinals\", \"statuses\", \"counts\"];\n\n",
    );
    out.push_str("/// The boundary of the review: this prose people write and read.\n");
    out.push_str("pub const PROSE_UNDER_REVIEW: &[&str] = &[\n");
    for field in [
        "title",
        "context",
        "decision",
        "trade_offs",
        "constraints",
        "goal",
        "input_contract",
        "output_contract",
        "invariants",
        "statement",
        "rationale",
        "angle",
        "findings",
    ] {
        let _ = writeln!(out, "    {field:?},");
    }
    out.push_str("];\n\n");

    // Номера: серии собираются из slug, порядок внутри серии — по числу.
    let mut numbered: Vec<(String, u16, &str)> = Vec::new();
    for entry in decisions.iter().chain(specs.iter()) {
        if let Some((series, number)) = ordinal_of(&entry.slug) {
            numbered.push((series, number, entry.slug.as_str()));
        }
    }
    numbered.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    out.push_str("/// Ordinal numbers of documents: the scan derives them from the slugs.\n");
    out.push_str("#[allow(non_upper_case_globals)]\npub mod ordinals {\n");
    for (_, number, slug) in &numbered {
        let ident = slug_ident(slug);
        let _ = writeln!(
            out,
            "    /// The ordinal of {slug} within its series.\n    pub const {ident}: u16 = {number};"
        );
    }
    let mut series_seen: Vec<String> = Vec::new();
    for (series, _, _) in &numbered {
        if series_seen.last().map(String::as_str) != Some(series.as_str()) {
            series_seen.push(series.clone());
        }
    }
    for series in &series_seen {
        let last = numbered
            .iter()
            .filter(|(s, _, _)| s == series)
            .map(|(_, n, _)| *n)
            .next_back()
            .unwrap_or(0);
        let upper = series.replace('-', "_").to_ascii_uppercase();
        let _ = writeln!(
            out,
            "    /// The next free ordinal of the {series} series.\n    pub const NEXT_{upper}: u16 = {};",
            last + 1
        );
    }
    for series in &series_seen {
        let numbers: Vec<u16> = numbered
            .iter()
            .filter(|(s, _, _)| s == series)
            .map(|(_, n, _)| *n)
            .collect();
        let upper = series.replace('-', "_").to_ascii_uppercase();
        let listed = numbers
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(
            out,
            "    /// Ordinal numbers of the {series} series, in order.\n    const SERIES_{upper}: &[u16] = &[{listed}];\n"
        );
    }
    if !series_seen.is_empty() {
        out.push_str("    /// Whether every next number follows the previous one.\n");
        out.push_str("    const fn contiguous(numbers: &[u16]) -> bool {\n");
        out.push_str("        let mut i = 1;\n        while i < numbers.len() {\n");
        out.push_str(
            "            if numbers[i - 1] + 1 != numbers[i] {\n                return false;\n            }\n            i += 1;\n        }\n        true\n    }\n\n",
        );
        out.push_str(
            "    // Contiguity is evaluated by the compiler: a gap or a duplicate among\n",
        );
        out.push_str("    // the numbers is a constant evaluation error (E0080).\n");
        for series in &series_seen {
            let upper = series.replace('-', "_").to_ascii_uppercase();
            let _ = writeln!(
                out,
                "    const _: () = assert!(contiguous(SERIES_{upper}), \"dacc-scan: the {series} series of ordinal numbers is not contiguous\");\n"
            );
        }
    }
    out.push_str("}\n\n");

    out.push_str(
        "/// Status classes of documents: the scan classifies every record by its text.\n",
    );
    out.push_str("#[allow(non_upper_case_globals)]\npub mod statuses {\n");
    out.push_str(
        "    /// The status class of a document.\n    #[derive(Debug, Clone, Copy, PartialEq, Eq)]\n    pub enum Status {\n",
    );
    out.push_str("        /// The record is a draft.\n        Draft,\n");
    out.push_str("        /// The record is active.\n        Active,\n");
    out.push_str("        /// The record is deprecated.\n        Deprecated,\n");
    out.push_str(
        "        /// The record is superseded by another one.\n        SupersededBy,\n    }\n\n",
    );
    out.push_str("    impl Status {\n");
    out.push_str(
        "        /// The numeric class code, compared in the constants below.\n        #[must_use]\n        pub const fn code(self) -> u8 {\n            match self {\n",
    );
    out.push_str(
        "                Self::Draft => 0,\n                Self::Active => 1,\n                Self::Deprecated => 2,\n                Self::SupersededBy => 3,\n            }\n        }\n\n",
    );
    out.push_str(
        "        /// The class of the status the compiler evaluated.\n        #[must_use]\n        pub const fn of(status: &::dacc_knowledge::DocStatus) -> Self {\n            match status {\n",
    );
    out.push_str(
        "                ::dacc_knowledge::DocStatus::Draft => Self::Draft,\n                ::dacc_knowledge::DocStatus::Active => Self::Active,\n",
    );
    out.push_str(
        "                ::dacc_knowledge::DocStatus::Deprecated => Self::Deprecated,\n                ::dacc_knowledge::DocStatus::SupersededBy(_) => Self::SupersededBy,\n            }\n        }\n    }\n\n",
    );
    for entry in decisions.iter().chain(specs.iter()) {
        let ident = slug_ident(&entry.slug);
        let slug = entry.slug.as_str();
        let class = class_name(entry.status);
        let _ = writeln!(
            out,
            "    /// The status of {slug}, classified by the scan.\n    pub const {ident}: Status = Status::{class};"
        );
    }
    out.push_str("}\n\n");

    out.push_str("// Scan cross-checked against the compiler for every class: the scan reads\n");
    out.push_str("// the text, the compiler evaluates the value. A divergence is a constant\n");
    out.push_str("// evaluation error (E0080).\n");
    for (entry, value) in decisions
        .iter()
        .map(|e| (e, "DECISION"))
        .chain(specs.iter().map(|e| (e, "SPEC")))
    {
        let ident = slug_ident(&entry.slug);
        let slug = entry.slug.as_str();
        let class = class_name(entry.status);
        let _ = writeln!(
            out,
            "const _: () = assert!(statuses::{ident}.code() == statuses::Status::of(&{ident}::{value}.status).code(), \"dacc-scan: by its text {slug} is {class}, the compiler evaluated otherwise\");"
        );
    }
    out.push('\n');

    out.push_str("/// Counters derived by the scan: counted over the registry, not by hand.\n");
    out.push_str("pub mod counts {\n");
    emit_counts(&mut out, "DECISIONS", "Decisions", decisions);
    emit_counts(&mut out, "SPECS", "Specifications", specs);
    out.push_str("}\n");
    out
}

/// Библиотечная точка входа раскладки реестра (решение 27): скан решений,
/// спецификаций, плана и журнала, порождение и запись всех файлов в `out_dir`.
/// build.rs потребителя сводится к одному вызову; каталоги реестра ожидаются в
/// корне `registry_dir` — он же корень крейта документов.
pub fn emit_registry(
    registry_dir: &Path,
    src_dirs: &[&Path],
    out_dir: &Path,
) -> Result<(), ScanError> {
    // Разметка кода нужна раньше остального: по ней сверяются инлайн-ссылки
    // прозы решений и спецификаций (решение 24).
    let anchors = anchors::scan_anchors(src_dirs)?;
    let anchor_ids: HashSet<&str> = anchors.iter().map(|a| a.id.as_str()).collect();
    check_inline_links_in_dir(&registry_dir.join("adr"), &anchor_ids)?;
    check_inline_links_in_dir(&registry_dir.join("rfc"), &anchor_ids)?;

    let decisions = scan_decisions(&registry_dir.join("adr"))?;
    let adr_refs = emit_refs(&decisions);
    check_bypass_patterns_live(&adr_refs)?;
    write_file(out_dir, "adr.rs", &adr_refs)?;

    let specs = scan_specs(&registry_dir.join("rfc"))?;
    write_file(out_dir, "rfc.rs", &emit_spec_refs(&specs))?;

    // Выводимое: номера, статусы и счётчики порождаются, а не пишутся руками
    // (работа w-derived-prose).
    write_file(out_dir, "derived.rs", &emit_derived(&decisions, &specs))?;

    let thrusts = plan::scan_plan(&registry_dir.join("thrust"), &plan::THRUSTS)?;
    let slices = plan::scan_plan(&registry_dir.join("slice"), &plan::SLICES)?;
    let work = plan::scan_plan(&registry_dir.join("work"), &plan::WORK)?;
    let obligations = plan::scan_plan(&registry_dir.join("obligation"), &plan::OBLIGATIONS)?;
    let limitations = plan::scan_plan(&registry_dir.join("limitation"), &plan::LIMITATIONS)?;
    let upgrades = plan::scan_plan(&registry_dir.join("upgrade"), &plan::UPGRADES)?;
    let invariants = plan::scan_plan(&registry_dir.join("invariant"), &plan::INVARIANTS)?;
    let reviews = plan::scan_plan(&registry_dir.join("review"), &plan::REVIEWS)?;
    let mut code = plan::emit_plan(&thrusts, &plan::THRUSTS);
    code.push_str(&plan::emit_plan(&slices, &plan::SLICES));
    code.push_str(&plan::emit_plan(&work, &plan::WORK));
    code.push_str(&plan::emit_plan(&obligations, &plan::OBLIGATIONS));
    code.push_str(&plan::emit_plan(&limitations, &plan::LIMITATIONS));
    code.push_str(&plan::emit_plan(&upgrades, &plan::UPGRADES));
    code.push_str(&plan::emit_plan(&invariants, &plan::INVARIANTS));
    code.push_str(&plan::emit_plan(&reviews, &plan::REVIEWS));
    code.push_str(&plan::emit_work_checks(&work));
    code.push_str(&plan::emit_toil_checks(&work));
    code.push_str(&plan::emit_obligation_checks(&obligations));
    code.push_str(&plan::emit_limitation_checks(&limitations));
    write_file(out_dir, "plan.rs", &code)?;

    let journal = journal::scan_journal(
        &registry_dir.join("journal.toml"),
        &registry_dir.join("journal"),
        &work,
    )?;
    write_file(out_dir, "journal.rs", &journal)?;

    write_file(out_dir, "commit.rs", &commit_constant())?;

    for dir in [
        "adr",
        "rfc",
        "thrust",
        "slice",
        "work",
        "obligation",
        "limitation",
        "upgrade",
        "invariant",
        "review",
        "journal",
        "journal.toml",
    ] {
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
        use dacc_knowledge::DocStatus;
        dacc_knowledge::adr!(
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

    /// Выводимое порождается сканом (работа w-derived-prose): номера со
    /// следующим свободным, статусы со сверкой с компилятором и счётчики —
    /// вместо того чтобы считать их вручную через git log и grep.
    #[test]
    fn emit_derived_derives_ordinals_statuses_and_counts() {
        let ds = vec![
            ScannedSlug {
                slug: "adr-2026-001".into(),
                status: Status::Active,
                file: "/x/adr-2026-001.rs".into(),
            },
            ScannedSlug {
                slug: "adr-2026-002".into(),
                status: Status::SupersededBy,
                file: "/x/adr-2026-002.rs".into(),
            },
        ];
        let specs = vec![ScannedSlug {
            slug: "rfc-2026-001".into(),
            status: Status::Active,
            file: "/x/rfc-2026-001.rs".into(),
        }];
        let out = emit_derived(&ds, &specs);
        assert!(out.contains("pub const adr_2026_002: u16 = 2;"), "{out}");
        assert!(out.contains("pub const NEXT_ADR_2026: u16 = 3;"), "{out}");
        assert!(out.contains("pub const NEXT_RFC_2026: u16 = 2;"), "{out}");
        assert!(
            out.contains("const SERIES_ADR_2026: &[u16] = &[1, 2];"),
            "{out}"
        );
        assert!(out.contains("contiguous(SERIES_ADR_2026)"), "{out}");
        assert!(
            out.contains("pub const adr_2026_002: Status = Status::SupersededBy;"),
            "{out}"
        );
        assert!(out.contains("adr_2026_002::DECISION.status"), "{out}");
        assert!(out.contains("rfc_2026_001::SPEC.status"), "{out}");
        assert!(
            out.contains("pub const DECISIONS_SUPERSEDED: usize = 1;"),
            "{out}"
        );
        assert!(out.contains("pub const SPECS_ACTIVE: usize = 1;"), "{out}");
        assert!(out.contains("pub const DERIVED_BY_SCAN"), "{out}");
        assert!(out.contains("pub const PROSE_UNDER_REVIEW"), "{out}");
    }

    /// Slug без числового номера серии его не порождает: номер живёт только
    /// там, где он есть в идентификаторе.
    #[test]
    fn emit_derived_leaves_unnumbered_slugs_without_ordinals() {
        let ds = vec![ScannedSlug {
            slug: "adr-direct-plan".into(),
            status: Status::Active,
            file: "/x/adr-direct-plan.rs".into(),
        }];
        let out = emit_derived(&ds, &[]);
        assert!(!out.contains("NEXT_"), "{out}");
        assert!(
            out.contains("pub const adr_direct_plan: Status = Status::Active;"),
            "{out}"
        );
    }

    /// Каталог реестра во временной папке: имя файла → содержимое.
    fn registry(tag: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("dacc-scan-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for (name, text) in files {
            fs::write(dir.join(name), text).unwrap();
        }
        dir
    }

    const ACTIVE: &str = "dacc_knowledge::adr!(status: DocStatus::Active,);";

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
        let text = "dacc_knowledge::adr!(status: DocStatus::SupersededBy(dacc_core::AdrRef::__from_scan(\"adr-2026-002\")),);";
        let err = parse_decision(text, "adr-2026-001").expect_err("ручной __from_scan принят");
        assert!(err.to_string().contains("__from_scan"), "{err}");
    }

    /// Решение 24: извлекается только форма `` `[id]` ``; свободное упоминание
    /// имени символа и иная скобочная форма ссылкой не являются.
    #[test]
    fn extracts_inline_links_and_ignores_free_mentions() {
        let text = "Форма `[plan-ir]` и свободное упоминание plan_ir, а ещё `[hdr]` и `[Plan-IR]`.";
        let links = extract_inline_links(text);
        assert_eq!(links, vec!["plan-ir".to_owned(), "hdr".to_owned()]);
    }

    /// Решение 24: ссылка на несуществующий якорь — отказ сборки.
    #[test]
    fn inline_link_to_unmarked_anchor_is_refused() {
        let anchors: HashSet<&str> = ["plan-ir"].into_iter().collect();
        let err = check_inline_links("Ссылка `[hdr]` не туда.", "a.rs", &anchors).unwrap_err();
        assert!(err.to_string().contains("hdr"), "{err}");
    }

    /// Решение 24: удаление разметки ломает ссылку — та же ссылка, которая
    /// резолвилась при разметке, становится отказом, когда якорь исчезает.
    #[test]
    fn removing_markup_breaks_the_link() {
        let with_anchor: HashSet<&str> = ["plan-ir"].into_iter().collect();
        assert!(check_inline_links("Ссылка `[plan-ir]`.", "a.rs", &with_anchor).is_ok());

        let without_anchor: HashSet<&str> = HashSet::new();
        let err = check_inline_links("Ссылка `[plan-ir]`.", "a.rs", &without_anchor).unwrap_err();
        assert!(err.to_string().contains("plan-ir"), "{err}");
    }

    /// Решение 24: свободное упоминание имени символа (без формы ссылки) не
    /// проверяется и отказа не даёт.
    #[test]
    fn free_mention_of_a_symbol_is_not_checked() {
        let anchors: HashSet<&str> = HashSet::new();
        assert!(
            check_inline_links("Упоминание plan_ir без формы ссылки.", "a.rs", &anchors).is_ok()
        );
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
        let work = vec![ScannedSlug {
            slug: "w-001".into(),

            status: Status::Draft,
            file: "/x/w-001.rs".into(),
        }];
        // Проверка до начала работы — нарушение автомата, закрытый срез при
        // незавершённой работе — константная проверка: оба пути порождают
        // сообщение, половина которого приходит из dacc-journal.
        let events = [
            dacc_journal::Event::new(
                dacc_journal::Subject::Work("w-001".to_owned()),
                "2026-09-11T03:15:01Z".to_owned(),
                dacc_journal::Kind::Gate {
                    gate: "commit".into(),
                    tree: "a".repeat(40),
                    verdict: dacc_journal::GateVerdict::Prose("ok".into()),
                },
            ),
            dacc_journal::Event::new(
                dacc_journal::Subject::Slice("s-003".to_owned()),
                "2026-09-11T03:15:02Z".to_owned(),
                dacc_journal::Kind::Closed,
            ),
        ];
        // Нечитаемый файл события: причина приходит из разбора формата.
        let unreadable = dacc_journal::parse_event("w-001/x.toml", "[table]\n").unwrap_err();
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
        let text = "dacc_knowledge::adr!(status: DocStatus::Active, context: r#\"вызов __from_scan руками запрещён\"#,);";
        assert!(parse_decision(text, "adr-2026-001").is_ok());
    }

    /// Anti-vacuity: порождённый код без конструктора скана — протухший
    /// паттерн поиска обхода, а не чистое дерево.
    #[test]
    fn a_dead_bypass_pattern_is_refused() {
        assert!(check_bypass_patterns_live("no constructor here").is_err());
        assert!(check_bypass_patterns_live("AdrRef::__from_scan(\"adr-x\")").is_ok());
    }

    /// Способ исправления есть у каждого нарушения: подсказка «как чинить»
    /// невыразимо отсутствует, и она входит в сообщение.
    #[test]
    fn every_scan_error_names_its_fix() {
        let errors: [ScanError; 7] = [
            ScanError::Io("x".into()),
            ScanError::Parse {
                file: "f.rs".into(),
                detail: "d".into(),
            },
            ScanError::IdMismatch {
                file: "f.rs".into(),
                declared: 1,
                expected: "x".into(),
            },
            ScanError::Bypass {
                file: "f.rs".into(),
                ident: "i".into(),
            },
            ScanError::Anchor {
                file: "f.rs".into(),
                line: 1,
                detail: "d".into(),
            },
            ScanError::InlineLink {
                file: "f.rs".into(),
                detail: "d".into(),
            },
            ScanError::DeadPattern { ident: "i".into() },
        ];
        for error in errors {
            assert!(!error.fix().trim().is_empty(), "{error}: fix is empty");
            assert!(error.to_string().contains("fix:"), "{}", error);
        }
    }

    /// Отрицательные сценарии — удалённый якорь, дубликат идентификатора и
    /// устаревшая ссылка — пойманы, и каждый называет способ исправления.
    #[test]
    fn negative_scenarios_name_the_fix() {
        // Удалённый якорь: инлайн-ссылка прозы на неразмеченный символ.
        let mut anchors = HashSet::new();
        anchors.insert("plan-ir");
        let err = check_inline_links("проза `[missing-anchor]`", "f.rs", &anchors).unwrap_err();
        assert!(err.to_string().contains("fix:"), "{err}");

        // Устаревшая ссылка: конструктор скана, вызванный руками в реестре.
        let text =
            "dacc_knowledge::adr!(status: DocStatus::SupersededBy(dacc_core::AdrRef::__from_scan(\"adr-2026-002\")),);";
        let err = parse_decision(text, "adr-x").unwrap_err();
        assert!(err.to_string().contains("fix:"), "{err}");

        // Дубликат идентификатора: один id разметки в двух местах.
        let dir = std::env::temp_dir().join(format!("dacc-neg-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("a.rs"),
            "#[doc_anchor(id = \"hdr\")]\npub struct A;\n",
        )
        .unwrap();
        fs::write(
            dir.join("b.rs"),
            "#[doc_anchor(id = \"hdr\")]\npub struct B;\n",
        )
        .unwrap();
        let err = anchors::scan_anchors(&[&dir]).unwrap_err();
        assert!(err.to_string().contains("fix:"), "{err}");
    }

    /// Точка входа раскладки (решение 27): пишет все порождённые файлы реестра.
    #[test]
    fn emit_registry_writes_all_generated_files() {
        let root = std::env::temp_dir().join(format!("dacc-registry-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for dir in [
            "adr",
            "rfc",
            "thrust",
            "slice",
            "work",
            "obligation",
            "limitation",
            "upgrade",
            "invariant",
            "review",
            "journal",
        ] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        fs::write(root.join("adr/adr-direct-plan.rs"), ACTIVE).unwrap();
        let out = root.join("out");
        fs::create_dir_all(&out).unwrap();

        emit_registry(&root, &[], &out).unwrap();

        for name in [
            "adr.rs",
            "rfc.rs",
            "derived.rs",
            "plan.rs",
            "journal.rs",
            "commit.rs",
        ] {
            assert!(out.join(name).is_file(), "{name} is missing");
        }
        let adr = fs::read_to_string(out.join("adr.rs")).unwrap();
        assert!(adr.contains("adr_direct_plan"), "{adr}");
    }
}
