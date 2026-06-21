use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;

const TITLE_WEIGHT: f64 = 3.0;
const CONCEPT_ID_WEIGHT: f64 = 2.5;
const TAG_WEIGHT: f64 = 1.5;
const DESCRIPTION_WEIGHT: f64 = 1.0;
const SEGMENT_EXACT_BONUS: f64 = 0.35;
const SEGMENT_PREFIX_BONUS: f64 = 0.15;

pub fn normalize_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

fn normalize_identifier(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}

fn field_match_score(field: &str, tokens: &[String]) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }

    let normalized_field = normalize_identifier(field);
    let matches = tokens
        .iter()
        .filter(|token| {
            let normalized_token = normalize_identifier(token);
            !normalized_token.is_empty()
                && (field.to_lowercase().contains(token.as_str())
                    || normalized_field.contains(&normalized_token))
        })
        .count();

    matches as f64 / tokens.len() as f64
}

fn segment_match_bonus(concept_id: &str, tokens: &[String]) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }

    let segments = concept_id.split('/').collect::<Vec<_>>();
    let mut bonus = 0.0;

    for token in tokens {
        let normalized_token = normalize_identifier(token);
        if normalized_token.is_empty() {
            continue;
        }

        for segment in &segments {
            let segment_lower = segment.to_lowercase();
            let normalized_segment = normalize_identifier(segment);
            if normalized_segment == normalized_token {
                bonus += SEGMENT_EXACT_BONUS;
            } else if normalized_segment.starts_with(&normalized_token)
                || normalized_token.starts_with(&normalized_segment)
            {
                bonus += SEGMENT_PREFIX_BONUS;
            } else if segment_lower.contains(token.as_str()) {
                bonus += SEGMENT_PREFIX_BONUS * 0.5;
            }
        }
    }

    bonus.min(1.0)
}

pub fn rank_okf_concept(page: &OkfConceptSummary, tokens: &[String]) -> f64 {
    if tokens.is_empty() {
        return 0.5;
    }

    let title_score = field_match_score(&page.title, tokens) * TITLE_WEIGHT;
    let concept_id_score = field_match_score(&page.concept_id, tokens) * CONCEPT_ID_WEIGHT;
    let tag_score = field_match_score(&page.tags.join(" "), tokens) * TAG_WEIGHT;
    let description_score = field_match_score(&page.description, tokens) * DESCRIPTION_WEIGHT;
    let weighted = (title_score + concept_id_score + tag_score + description_score)
        / (TITLE_WEIGHT + CONCEPT_ID_WEIGHT + TAG_WEIGHT + DESCRIPTION_WEIGHT);
    let segment_bonus = segment_match_bonus(&page.concept_id, tokens);
    let base_score = weighted + segment_bonus;
    let recency_bonus = if base_score > 0.0 {
        page.source_count as f64 * 0.01
    } else {
        0.0
    };

    (base_score + recency_bonus).clamp(0.0, 1.5)
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
            .then_with(|| left.1.concept_id.cmp(&right.1.concept_id))
    });

    ranked.truncate(top_k.max(1) as usize);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;

    fn sample_concept(concept_id: &str, title: &str, description: &str) -> OkfConceptSummary {
        OkfConceptSummary {
            title: title.to_string(),
            concept_id: concept_id.to_string(),
            concept_type: "Knowledge Concept".to_string(),
            logical_path: format!("okf/{concept_id}.md"),
            bundle_relative_path: format!("{concept_id}.md"),
            description: description.to_string(),
            source_count: 1,
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            tags: vec!["demo".to_string()],
        }
    }

    #[test]
    fn rank_okf_concepts_matches_concept_id_tokens() {
        let pages = vec![
            sample_concept("tables/users", "Users Table", "User dimension table"),
            sample_concept("tables/votes", "Votes Table", "Vote fact table"),
        ];

        let ranked = rank_okf_concepts(pages, "users", 5);

        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].1.concept_id, "tables/users");
    }

    #[test]
    fn rank_okf_concepts_prefers_title_match_over_description_only() {
        let pages = vec![
            sample_concept("tables/users", "Users Table", "User dimension table"),
            sample_concept(
                "tables/posts",
                "Posts Table",
                "Contains users activity history",
            ),
        ];

        let ranked = rank_okf_concepts(pages, "users", 5);

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].1.concept_id, "tables/users");
        assert!(ranked[0].0 > ranked[1].0);
    }

    #[test]
    fn rank_okf_concepts_treats_hyphen_and_underscore_as_equivalent() {
        let pages = vec![sample_concept(
            "posts/tag_wiki",
            "Tag Wiki",
            "Tag metadata concept",
        )];

        let ranked = rank_okf_concepts(pages, "tag-wiki", 5);

        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].1.concept_id, "posts/tag_wiki");
    }
}
