use sdkwork_knowledgebase_contract::wiki::{WikiPageSummary, WikiPageType};

pub fn render_index_md(space_name: &str, pages: &[WikiPageSummary]) -> String {
    let mut output =
        format!("# Index\n\n## Overview\n\n- [[Overview]] - Current synthesis for {space_name}.\n");

    for (heading, page_type) in [
        ("Sources", WikiPageType::Source),
        ("Entities", WikiPageType::Entity),
        ("Concepts", WikiPageType::Concept),
        ("Topics", WikiPageType::Topic),
        ("Answers", WikiPageType::Answer),
        ("Comparisons", WikiPageType::Comparison),
        ("Open Questions", WikiPageType::Index),
    ] {
        output.push_str("\n## ");
        output.push_str(heading);
        output.push_str("\n\n");

        let mut wrote_any = false;
        for page in pages.iter().filter(|page| page.page_type == page_type) {
            wrote_any = true;
            output.push_str("- [[");
            output.push_str(&page.title);
            output.push_str("]] - ");
            output.push_str(&page.summary);
            output.push_str(" sources: ");
            output.push_str(&page.source_count.to_string());
            output.push_str(", updated: ");
            output.push_str(&page.updated_at);
            output.push('\n');
        }

        if !wrote_any {
            output.push_str("- None.\n");
        }
    }

    output
}
