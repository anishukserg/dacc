//! `cargo dacc map [--format json]` — карта реестра одной командой: дерево
//! направлений, срезов и работ со свёрткой журнала и счётчиками.

use super::form::{cap_text, json_string, xml_string, MAP_LIMIT};
use super::{count_files, records, scan_args, Work};
use crate::format::Format;
use crate::work::{stage_text, usage, Context, Refusal};
use crate::{code, record};
use std::ffi::OsString;

/// Направление карты.
struct Thrust {
    id: String,
    title: String,
    outcome: String,
}

/// Срез карты.
struct Slice {
    id: String,
    title: String,
    outcome: String,
    thrust: String,
    closed: bool,
}

pub(super) fn run(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "map [--format json|xml]")?;
    if scan.position.is_some() || scan.status.is_some() || scan.file.is_some() || scan.check {
        return Err(usage(code::USAGE, "map [--format json|xml]"));
    }
    let context = Context::open()?;
    let root = &context.repo.root;

    let mut thrusts = Vec::new();
    for (id, text) in records(&root.join(format!("{}/thrust", context.repo.config.doc))) {
        thrusts.push(Thrust {
            outcome: record::field_string(&text, "outcome").unwrap_or_default(),
            title: record::title(&text),
            id,
        });
    }

    let mut slices = Vec::new();
    for (id, text) in records(&root.join(context.repo.config.slice_dir())) {
        slices.push(Slice {
            closed: context.journal.closed_slices.contains_key(&id),
            thrust: record::field_slug(&text, "thrust").unwrap_or_default(),
            outcome: record::field_string(&text, "outcome").unwrap_or_default(),
            title: record::title(&text),
            id,
        });
    }

    let mut works = Vec::new();
    for (id, record) in context.works()? {
        works.push(Work {
            state: stage_text(context.journal.stage(&id)).to_owned(),
            slice: record.slice.clone(),
            taxon: record.area.clone(),
            radius: record.radius.clone(),
            title: record.title,
            id,
        });
    }

    let decisions = count_files(&root.join(format!("{}/adr", context.repo.config.doc)));
    let specifications = count_files(&root.join(format!("{}/rfc", context.repo.config.doc)));

    match scan.format {
        Format::Json => {
            println!(
                "{}",
                render_json(&thrusts, &slices, &works, decisions, specifications)
            );
        }
        Format::Text => {
            print!(
                "{}",
                cap_text(
                    &render_text(&thrusts, &slices, &works, decisions, specifications),
                    MAP_LIMIT
                )
            );
        }
        Format::Xml => {
            println!(
                "{}",
                render_xml(&thrusts, &slices, &works, decisions, specifications)
            );
        }
    }
    Ok(0)
}

/// Карта текстом: дерево направлений, срезов и работ и счётчики.
fn render_text(
    thrusts: &[Thrust],
    slices: &[Slice],
    works: &[Work],
    decisions: usize,
    specifications: usize,
) -> String {
    let mut out = String::new();
    for thrust in thrusts {
        out.push_str(&format!("{} — {}\n", thrust.id, thrust.title));
        out.push_str(&format!("    {}\n", thrust.outcome));
        for slice in slices.iter().filter(|slice| slice.thrust == thrust.id) {
            let state = if slice.closed { "closed" } else { "open" };
            out.push_str(&format!("  {} [{}] — {}\n", slice.id, state, slice.title));
            out.push_str(&format!("      {}\n", slice.outcome));
            for work in works
                .iter()
                .filter(|work| work.slice.as_deref() == Some(slice.id.as_str()))
            {
                out.push_str(&format!(
                    "    {} [{}] — {}\n",
                    work.id, work.state, work.title
                ));
            }
        }
    }
    let open = slices.iter().filter(|slice| !slice.closed).count();
    out.push_str(&format!(
        "thrusts: {}, slices: {} (open {}, closed {}), works: {}\n",
        thrusts.len(),
        slices.len(),
        open,
        slices.len() - open,
        works.len()
    ));
    let mut stages: Vec<String> = Vec::new();
    for state in crate::work::STAGE_TEXTS {
        let count = works.iter().filter(|work| work.state == state).count();
        if count > 0 {
            stages.push(format!("{state} {count}"));
        }
    }
    out.push_str(&format!("works by state: {}\n", stages.join(", ")));
    out.push_str(&format!(
        "decisions: {decisions}, specifications: {specifications}\n"
    ));
    out
}

/// Карта машинным форматом: те же данные полями. Усечение — до сериализации:
/// записи добавляются до потолка MAP_LIMIT, хвост со счётчиками всегда
/// влезает, и поле truncated названо явно (RFC-0003). Готовый текст не
/// разбирается и не переписывается — форма ответа не зависит от порядка ключей.
fn render_json(
    thrusts: &[Thrust],
    slices: &[Slice],
    works: &[Work],
    decisions: usize,
    specifications: usize,
) -> String {
    let mut thrust_entries: Vec<String> = Vec::new();
    for thrust in thrusts {
        thrust_entries.push(format!(
            "{{\"id\": {}, \"title\": {}, \"outcome\": {}}}",
            json_string(&thrust.id),
            json_string(&thrust.title),
            json_string(&thrust.outcome)
        ));
    }
    let mut slice_entries: Vec<String> = Vec::new();
    for slice in slices {
        slice_entries.push(format!(
            "{{\"id\": {}, \"title\": {}, \"thrust\": {}, \"closed\": {}, \"outcome\": {}}}",
            json_string(&slice.id),
            json_string(&slice.title),
            json_string(&slice.thrust),
            slice.closed,
            json_string(&slice.outcome)
        ));
    }
    let mut work_entries: Vec<String> = Vec::new();
    for work in works {
        work_entries.push(format!(
            "{{\"id\": {}, \"title\": {}, \"slice\": {}, \"taxon\": {}, \"radius\": {}, \"state\": {}}}",
            json_string(&work.id),
            json_string(&work.title),
            work.slice
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            work.taxon
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            work.radius
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            json_string(&work.state)
        ));
    }

    let mut out = String::from("{\"schema\": \"dacc-map\", \"thrusts\": [");
    let mut truncated = false;
    for (index, entry) in thrust_entries.iter().enumerate() {
        if out.len() + entry.len() + 256 > MAP_LIMIT {
            truncated = true;
            break;
        }
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(entry);
    }
    out.push_str("], \"slices\": [");
    for (index, entry) in slice_entries.iter().enumerate() {
        if truncated || out.len() + entry.len() + 256 > MAP_LIMIT {
            truncated = true;
            break;
        }
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(entry);
    }
    out.push_str("], \"works\": [");
    for (index, entry) in work_entries.iter().enumerate() {
        if truncated || out.len() + entry.len() + 256 > MAP_LIMIT {
            truncated = true;
            break;
        }
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(entry);
    }
    out.push_str(&format!(
        "], \"decisions\": {decisions}, \"specifications\": {specifications}, \"truncated\": {truncated}}}\n"
    ));
    out
}

/// Карта версионированным контрактом XML (RFC-0003): тот же потолок и явное
/// усечение в элементе counts.
fn render_xml(
    thrusts: &[Thrust],
    slices: &[Slice],
    works: &[Work],
    decisions: usize,
    specifications: usize,
) -> String {
    let mut entries: Vec<String> = Vec::new();
    for thrust in thrusts {
        entries.push(format!(
            "  <thrust id=\"{}\" title=\"{}\"/>\n",
            xml_string(&thrust.id),
            xml_string(&thrust.title)
        ));
    }
    for slice in slices {
        entries.push(format!(
            "  <slice id=\"{}\" title=\"{}\" thrust=\"{}\" closed=\"{}\"/>\n",
            xml_string(&slice.id),
            xml_string(&slice.title),
            xml_string(&slice.thrust),
            slice.closed
        ));
    }
    for work in works {
        entries.push(format!(
            "  <work id=\"{}\" title=\"{}\" state=\"{}\"/>\n",
            xml_string(&work.id),
            xml_string(&work.title),
            xml_string(&work.state)
        ));
    }
    let total = entries.len();
    let head = String::from("<map schema=\"dacc-map\" version=\"1\">\n");
    let mut body = String::new();
    let mut shown = 0;
    for (index, entry) in entries.iter().enumerate() {
        if head.len() + body.len() + entry.len() + 128 > MAP_LIMIT {
            break;
        }
        body.push_str(entry);
        shown = index + 1;
    }
    format!(
        "{head}{body}  <counts decisions=\"{decisions}\" specifications=\"{specifications}\" shown=\"{shown}\" total=\"{total}\" truncated=\"{}\"/>\n</map>\n",
        shown < total
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work(id: &str, state: &str) -> Work {
        Work {
            id: id.to_owned(),
            title: format!("Работа {id}"),
            slice: None,
            taxon: None,
            radius: None,
            state: state.to_owned(),
        }
    }

    /// Счётчик карты ведёт каждую стадию свёртки, включая landed from
    /// history: пропущенная стадия краснеет (работа w-access-answers).
    #[test]
    fn map_counters_name_every_stage() {
        let works: Vec<Work> = crate::work::STAGE_TEXTS
            .iter()
            .enumerate()
            .map(|(index, stage)| work(&format!("w-{index:03}"), stage))
            .collect();
        let text = render_text(&[], &[], &works, 0, 0);
        for stage in crate::work::STAGE_TEXTS {
            assert!(text.contains(&format!("{stage} 1")), "{text}");
        }
        assert!(text.contains("works: 5"), "{text}");
    }

    /// Дерево карты собирается по slug-ссылкам записей: идентификатор пути
    /// `crate::thrust::t_001` ведёт к записи файла `t-001.rs`, и срезы и
    /// работы стоят под своим направлением (работа w-access-split).
    #[test]
    fn map_tree_matches_ident_refs_to_slugs() {
        let thrusts = vec![Thrust {
            id: "t-001".to_owned(),
            title: "Первое".to_owned(),
            outcome: "Исход.".to_owned(),
        }];
        let slices = vec![Slice {
            id: "s-001".to_owned(),
            title: "Срез".to_owned(),
            outcome: "Исход.".to_owned(),
            thrust: record::slug_of("crate::thrust::t_001").unwrap_or_default(),
            closed: false,
        }];
        let works = vec![work("w-001", "planned")];
        let text = render_text(&thrusts, &slices, &works, 0, 0);
        let at = text.find("  s-001").expect("срез под направлением");
        assert!(text[..at].contains("t-001"), "{text}");
    }
}
