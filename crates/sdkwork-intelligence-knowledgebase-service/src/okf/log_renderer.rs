use sdkwork_knowledgebase_contract::okf::{OkfLogEntry, OkfLogEventType};
use std::collections::BTreeMap;

pub fn render_log_md(entries: &[OkfLogEntry]) -> String {
    let mut output = String::from("# Log\n");
    if entries.is_empty() {
        output.push_str("\n_(empty)_\n");
        return output;
    }

    let mut grouped: BTreeMap<String, Vec<&OkfLogEntry>> = BTreeMap::new();
    for entry in entries {
        grouped
            .entry(log_date(&entry.occurred_at))
            .or_default()
            .push(entry);
    }

    for (date, day_entries) in grouped.into_iter().rev() {
        output.push_str("\n## ");
        output.push_str(&date);
        output.push('\n');
        for entry in day_entries {
            output.push_str("* **");
            output.push_str(event_label(entry.event_type));
            output.push_str("**: ");
            output.push_str(&entry.title);
            if !entry.affected_concepts.is_empty() {
                output.push_str(" (");
                output.push_str(&entry.affected_concepts.join(", "));
                output.push(')');
            }
            output.push('\n');
        }
    }

    output
}

fn log_date(timestamp: &str) -> String {
    timestamp
        .split('T')
        .next()
        .unwrap_or(timestamp)
        .to_string()
}

fn event_label(event_type: OkfLogEventType) -> &'static str {
    match event_type {
        OkfLogEventType::Publish => "Publish",
        OkfLogEventType::Ingest => "Creation",
        OkfLogEventType::Query => "Update",
        OkfLogEventType::FiledAnswer => "Update",
        OkfLogEventType::Compile => "Update",
        OkfLogEventType::Review => "Update",
        OkfLogEventType::Lint => "Update",
        OkfLogEventType::Eval => "Update",
        OkfLogEventType::Package => "Update",
        OkfLogEventType::Mirror => "Update",
        OkfLogEventType::DeltaUpdate => "Update",
    }
}
