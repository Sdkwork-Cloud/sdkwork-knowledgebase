use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;

pub fn normalize_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn rank_okf_concept(page: &OkfConceptSummary, tokens: &[String]) -> f64 {
    if tokens.is_empty() {
        return 0.5;
    }

    let haystack = format!(
        "{} {} {}",
        page.title.to_lowercase(),
        page.description.to_lowercase(),
        page.tags.join(" ").to_lowercase()
    );

    let matches = tokens
        .iter()
        .filter(|token| haystack.contains(token.as_str()))
        .count();

    matches as f64 / tokens.len() as f64
}

pub fn rank_okf_concepts(
    pages: Vec<OkfConceptSummary>,
    query: &str,
    top_k: u32,
) -> Vec<(f64, OkfConceptSummary)> {
    let normalized_query = normalize_query(query);
    let mut ranked = pages
        .into_iter()
        .map(|page| (rank_okf_concept(&page, &normalized_query), page))
        .filter(|(score, _)| *score > 0.0 || normalized_query.is_empty())
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ranked.truncate(top_k.max(1) as usize);
    ranked
}
