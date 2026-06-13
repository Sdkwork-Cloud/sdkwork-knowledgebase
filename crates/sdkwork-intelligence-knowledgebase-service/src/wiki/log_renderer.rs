use sdkwork_knowledgebase_contract::wiki::WikiLogEntry;

pub fn render_log_md(entries: &[WikiLogEntry]) -> String {
    let mut output = String::from("# Log\n");

    for entry in entries {
        output.push_str("\n## [");
        output.push_str(&entry.occurred_at);
        output.push_str("] ");
        output.push_str(entry.event_type.as_str());
        output.push_str(" | ");
        output.push_str(&entry.title);
        output.push('\n');
        output.push_str("- actor: ");
        output.push_str(&entry.actor);
        output.push('\n');

        if let Some(audit_event_id) = &entry.audit_event_id {
            output.push_str("- auditEventId: ");
            output.push_str(audit_event_id);
            output.push('\n');
        }

        if !entry.affected_pages.is_empty() {
            output.push_str("- affectedPages: ");
            output.push_str(&entry.affected_pages.join(", "));
            output.push('\n');
        }

        for warning in &entry.warnings {
            output.push_str("- warning: ");
            output.push_str(warning);
            output.push('\n');
        }
    }

    output
}
