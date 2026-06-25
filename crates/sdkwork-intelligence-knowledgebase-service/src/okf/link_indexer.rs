use crate::okf::document::{extract_concept_links, OkfConceptLink};

pub fn index_concept_links(
    body: &str,
    from_concept_id: &str,
    known_concept_ids: &[String],
) -> Vec<OkfConceptLink> {
    extract_concept_links(body, from_concept_id, known_concept_ids)
}
