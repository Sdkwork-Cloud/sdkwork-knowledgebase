use std::sync::Arc;

use crate::api::paths::backend_path;
use crate::api::paths::append_query_string;
use crate::http::{SdkworkError, SdkworkHttpClient};
use crate::models::{AnonymizeKnowledgeAuditSubjectRequest, ComplianceAuditEventsAnonymizeActorCreateResponse201, ComplianceAuditEventsExportCreateResponse201, CreateKnowledgeSourceRequest, ExportKnowledgeAuditEventsRequest, IndexesCreateResponse201, IndexesListResponse, IndexesRebuildResponse, IndexesRetrieveResponse, KnowledgeIndexRequest, KnowledgeOkfProfileRequest, KnowledgeRetrievalProfileRequest, OkfBundleExportCreateResponse201, OkfBundleExportRequest, OkfBundleExportRetrieveResponse, OkfBundleFilesListResponse, OkfBundleImportCreateResponse201, OkfBundleImportRequest, OkfBundleIndexCreateResponse201, OkfBundleIndexRebuildRequest, OkfCandidateReviewRequest, OkfCandidatesApproveResponse, OkfCandidatesListResponse, OkfCandidatesRejectResponse, OkfCompileJobRequest, OkfCompileJobsCreateResponse201, OkfConceptPublishRequest, OkfConceptsPublishResponse, OkfEvalRunsCreateResponse201, OkfLintRunsCreateResponse201, OkfLogEntriesCreateResponse201, OkfLogEntry, OkfProfileCreateResponse201, OkfProfileUpdateResponse, OkfQualityRunRequest, ProviderHealthListResponse, RetrievalProfilesCreateResponse201, RetrievalProfilesRetrieveResponse, RetrievalProfilesUpdateResponse, RetrievalTracesListResponse, RetrievalTracesRetrieveResponse, SourcesCreateResponse201, SourcesListResponse, SpacesListResponse, SpacesMembersListResponse, TenantsCurrentListResponse};

#[derive(Clone)]
pub struct KnowledgeApi {
    client: Arc<SdkworkHttpClient>,
}

impl KnowledgeApi {
    pub fn new(client: Arc<SdkworkHttpClient>) -> Self {
        Self { client }
    }

    /// List knowledge sources
    pub async fn sources_list(&self, cursor: Option<&str>, page_size: Option<i64>) -> Result<SourcesListResponse, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/knowledge/sources".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// Create a knowledge source
    pub async fn sources_create(&self, body: &CreateKnowledgeSourceRequest) -> Result<SourcesCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/sources".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Create an OKF compile job
    pub async fn okf_compile_jobs_create(&self, body: &OkfCompileJobRequest) -> Result<OkfCompileJobsCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/okf/compile_jobs".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// List OKF candidates
    pub async fn okf_candidates_list(&self, space_id: i64, cursor: Option<&str>, page_size: Option<i64>) -> Result<OkfCandidatesListResponse, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("spaceId", space_id, "form", true, false, None),
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/knowledge/okf/candidates".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// Approve an OKF candidate
    pub async fn okf_candidates_approve(&self, candidate_id: i64, body: &OkfCandidateReviewRequest) -> Result<OkfCandidatesApproveResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/okf/candidates/{}/approve", serialize_path_parameter(candidate_id, PathParameterSpec::new("candidateId", "simple", false))));
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Reject an OKF candidate
    pub async fn okf_candidates_reject(&self, candidate_id: i64, body: &OkfCandidateReviewRequest) -> Result<OkfCandidatesRejectResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/okf/candidates/{}/reject", serialize_path_parameter(candidate_id, PathParameterSpec::new("candidateId", "simple", false))));
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Publish an OKF concept
    pub async fn okf_concepts_publish(&self, concept_id: i64, body: &OkfConceptPublishRequest) -> Result<OkfConceptsPublishResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/okf/concepts/{}/publish", serialize_path_parameter(concept_id, PathParameterSpec::new("conceptId", "simple", false))));
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Create an OKF profile
    pub async fn okf_profile_create(&self, body: &KnowledgeOkfProfileRequest) -> Result<OkfProfileCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/okf/profile".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Update an OKF profile
    pub async fn okf_profile_update(&self, profile_id: i64, body: &KnowledgeOkfProfileRequest) -> Result<OkfProfileUpdateResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/okf/profile/{}", serialize_path_parameter(profile_id, PathParameterSpec::new("profileId", "simple", false))));
        self.client.patch(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Rebuild the OKF bundle index
    pub async fn okf_bundle_index_create(&self, body: &OkfBundleIndexRebuildRequest) -> Result<OkfBundleIndexCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/okf/index/rebuild".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Create an OKF log entry
    pub async fn okf_log_entries_create(&self, body: &OkfLogEntry) -> Result<OkfLogEntriesCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/okf/log_entries".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Create an OKF bundle export
    pub async fn okf_bundle_export_create(&self, body: &OkfBundleExportRequest) -> Result<OkfBundleExportCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/okf/exports".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Retrieve an OKF bundle export
    pub async fn okf_bundle_export_retrieve(&self, export_id: i64) -> Result<OkfBundleExportRetrieveResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/okf/exports/{}", serialize_path_parameter(export_id, PathParameterSpec::new("exportId", "simple", false))));
        self.client.get(&path, None, None).await
    }

    /// List OKF bundle files
    pub async fn okf_bundle_files_list(&self, cursor: Option<&str>, page_size: Option<i64>) -> Result<OkfBundleFilesListResponse, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/knowledge/okf/bundle/files".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// Create an OKF lint run
    pub async fn okf_lint_runs_create(&self, body: &OkfQualityRunRequest) -> Result<OkfLintRunsCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/okf/lint_runs".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Create an OKF eval run
    pub async fn okf_eval_runs_create(&self, body: &OkfQualityRunRequest) -> Result<OkfEvalRunsCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/okf/eval_runs".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Create a knowledge index
    pub async fn indexes_create(&self, body: &KnowledgeIndexRequest) -> Result<IndexesCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/indexes".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// List knowledge indexes
    pub async fn indexes_list(&self, cursor: Option<&str>, page_size: Option<i64>) -> Result<IndexesListResponse, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/knowledge/indexes".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// Retrieve a knowledge index
    pub async fn indexes_retrieve(&self, index_id: &str) -> Result<IndexesRetrieveResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/indexes/{}", serialize_path_parameter(index_id, PathParameterSpec::new("indexId", "simple", false))));
        self.client.get(&path, None, None).await
    }

    /// Rebuild a knowledge index
    pub async fn indexes_rebuild(&self, index_id: &str, body: &OkfBundleIndexRebuildRequest) -> Result<IndexesRebuildResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/indexes/{}/rebuild", serialize_path_parameter(index_id, PathParameterSpec::new("indexId", "simple", false))));
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Create a retrieval profile
    pub async fn retrieval_profiles_create(&self, body: &KnowledgeRetrievalProfileRequest) -> Result<RetrievalProfilesCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/retrieval_profiles".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Retrieve a retrieval profile
    pub async fn retrieval_profiles_retrieve(&self, profile_id: &str) -> Result<RetrievalProfilesRetrieveResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/retrieval_profiles/{}", serialize_path_parameter(profile_id, PathParameterSpec::new("profileId", "simple", false))));
        self.client.get(&path, None, None).await
    }

    /// Update a retrieval profile
    pub async fn retrieval_profiles_update(&self, profile_id: &str, body: &KnowledgeRetrievalProfileRequest) -> Result<RetrievalProfilesUpdateResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/retrieval_profiles/{}", serialize_path_parameter(profile_id, PathParameterSpec::new("profileId", "simple", false))));
        self.client.patch(&path, Some(body), None, None, Some("application/json")).await
    }

    /// List retrieval traces
    pub async fn retrieval_traces_list(&self, cursor: Option<&str>, page_size: Option<i64>) -> Result<RetrievalTracesListResponse, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/knowledge/retrieval_traces".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// Retrieve a retrieval trace
    pub async fn retrieval_traces_retrieve(&self, trace_id: &str) -> Result<RetrievalTracesRetrieveResponse, SdkworkError> {
        let path = backend_path(&format!("/knowledge/retrieval_traces/{}", serialize_path_parameter(trace_id, PathParameterSpec::new("traceId", "simple", false))));
        self.client.get(&path, None, None).await
    }

    /// Retrieve provider health status
    pub async fn provider_health_list(&self) -> Result<ProviderHealthListResponse, SdkworkError> {
        let path = backend_path(&"/knowledge/provider_health".to_string());
        self.client.get(&path, None, None).await
    }

    /// Retrieve current tenant knowledgebase status
    pub async fn tenants_current_list(&self) -> Result<TenantsCurrentListResponse, SdkworkError> {
        let path = backend_path(&"/knowledge/tenants/current".to_string());
        self.client.get(&path, None, None).await
    }

    /// Import an OKF bundle from drive staging
    pub async fn okf_bundle_import_create(&self, body: &OkfBundleImportRequest) -> Result<OkfBundleImportCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/okf/imports".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// List knowledge spaces
    pub async fn spaces_list(&self, cursor: Option<&str>, page_size: Option<i64>) -> Result<SpacesListResponse, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/knowledge/spaces".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// List knowledge space members
    pub async fn spaces_members_list(&self, space_id: &str, cursor: Option<&str>, page_size: Option<i64>) -> Result<SpacesMembersListResponse, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&format!("/knowledge/spaces/{}/members", serialize_path_parameter(space_id, PathParameterSpec::new("spaceId", "simple", false)))), &query);
        self.client.get(&path, None, None).await
    }

    /// Export knowledge audit events for a subject
    pub async fn compliance_audit_events_export_create(&self, body: &ExportKnowledgeAuditEventsRequest) -> Result<ComplianceAuditEventsExportCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/compliance/audit_events/export".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Anonymize audit events for a subject
    pub async fn compliance_audit_events_anonymize_actor_create(&self, body: &AnonymizeKnowledgeAuditSubjectRequest) -> Result<ComplianceAuditEventsAnonymizeActorCreateResponse201, SdkworkError> {
        let path = backend_path(&"/knowledge/compliance/audit_events/anonymize_actor".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

}

struct PathParameterSpec<'a> {
    name: &'a str,
    style: &'a str,
    explode: bool,
}

impl<'a> PathParameterSpec<'a> {
    fn new(name: &'a str, style: &'a str, explode: bool) -> Self {
        Self { name, style, explode }
    }
}

fn serialize_path_parameter<T: serde::Serialize>(value: T, spec: PathParameterSpec<'_>) -> String {
    let value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    if value.is_null() {
        return String::new();
    }
    let style = if spec.style.is_empty() { "simple" } else { spec.style };
    match value {
        serde_json::Value::Array(values) => serialize_path_array(spec.name, &values, style, spec.explode),
        serde_json::Value::Object(values) => serialize_path_object(spec.name, &values, style, spec.explode),
        value => format!("{}{}", path_primitive_prefix(spec.name, style), percent_encode(&primitive_to_string(&value))),
    }
}

fn serialize_path_array(name: &str, values: &[serde_json::Value], style: &str, explode: bool) -> String {
    let serialized = values
        .iter()
        .filter(|value| !value.is_null())
        .map(|value| percent_encode(&primitive_to_string(value)))
        .collect::<Vec<_>>();
    if serialized.is_empty() {
        return path_prefix(name, style);
    }
    if style == "matrix" {
        if explode {
            return serialized.iter().map(|item| format!(";{}={}", name, item)).collect::<Vec<_>>().join("");
        }
        return format!(";{}={}", name, serialized.join(","));
    }
    let separator = if explode { "." } else { "," };
    format!("{}{}", path_prefix(name, style), serialized.join(separator))
}

fn serialize_path_object(
    name: &str,
    values: &serde_json::Map<String, serde_json::Value>,
    style: &str,
    explode: bool,
) -> String {
    let mut entries = Vec::new();
    let mut exploded = Vec::new();
    for (key, value) in values {
        if value.is_null() {
            continue;
        }
        let escaped_key = percent_encode(key);
        let escaped_value = percent_encode(&primitive_to_string(value));
        if explode {
            if style == "matrix" {
                exploded.push(format!(";{}={}", escaped_key, escaped_value));
            } else {
                exploded.push(format!("{}={}", escaped_key, escaped_value));
            }
        } else {
            entries.push(escaped_key);
            entries.push(escaped_value);
        }
    }
    if style == "matrix" {
        if explode {
            return exploded.join("");
        }
        return format!(";{}={}", name, entries.join(","));
    }
    if explode {
        let separator = if style == "label" { "." } else { "," };
        return format!("{}{}", path_prefix(name, style), exploded.join(separator));
    }
    format!("{}{}", path_prefix(name, style), entries.join(","))
}

fn path_prefix(name: &str, style: &str) -> String {
    match style {
        "label" => ".".to_string(),
        "matrix" => format!(";{}", name),
        _ => String::new(),
    }
}

fn path_primitive_prefix(name: &str, style: &str) -> String {
    if style == "matrix" {
        format!(";{}=", name)
    } else {
        path_prefix(name, style)
    }
}


struct QueryParameterSpec<'a> {
    name: &'a str,
    value: serde_json::Value,
    style: &'a str,
    explode: bool,
    allow_reserved: bool,
    content_type: Option<&'a str>,
}

impl<'a> QueryParameterSpec<'a> {
    fn new<T: serde::Serialize>(
        name: &'a str,
        value: T,
        style: &'a str,
        explode: bool,
        allow_reserved: bool,
        content_type: Option<&'a str>,
    ) -> Self {
        Self {
            name,
            value: serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
            style,
            explode,
            allow_reserved,
            content_type,
        }
    }
}

fn build_query_string(parameters: &[QueryParameterSpec<'_>]) -> String {
    let mut pairs = Vec::new();
    for parameter in parameters {
        append_serialized_parameter(&mut pairs, parameter);
    }
    pairs.join("&")
}

fn append_serialized_parameter(pairs: &mut Vec<String>, parameter: &QueryParameterSpec<'_>) {
    if parameter.value.is_null() {
        return;
    }
    if parameter.content_type.is_some() {
        pairs.push(format!(
            "{}={}",
            percent_encode(parameter.name),
            encode_query_value(&parameter.value.to_string(), parameter.allow_reserved)
        ));
        return;
    }

    let style = if parameter.style.is_empty() { "form" } else { parameter.style };
    match &parameter.value {
        serde_json::Value::Array(values) => append_array_parameter(pairs, parameter.name, values, style, parameter.explode, parameter.allow_reserved),
        serde_json::Value::Object(values) if style == "deepObject" => append_deep_object_parameter(pairs, parameter.name, values, parameter.allow_reserved),
        serde_json::Value::Object(values) => append_object_parameter(pairs, parameter.name, values, style, parameter.explode, parameter.allow_reserved),
        value => pairs.push(format!("{}={}", percent_encode(parameter.name), encode_query_value(&primitive_to_string(value), parameter.allow_reserved))),
    }
}

fn append_array_parameter(
    pairs: &mut Vec<String>,
    name: &str,
    values: &[serde_json::Value],
    style: &str,
    explode: bool,
    allow_reserved: bool,
) {
    let serialized = values.iter().filter(|value| !value.is_null()).map(primitive_to_string).collect::<Vec<_>>();
    if serialized.is_empty() {
        return;
    }
    if style == "form" && explode {
        for item in serialized {
            pairs.push(format!("{}={}", percent_encode(name), encode_query_value(&item, allow_reserved)));
        }
        return;
    }
    pairs.push(format!("{}={}", percent_encode(name), encode_query_value(&serialized.join(","), allow_reserved)));
}

fn append_object_parameter(
    pairs: &mut Vec<String>,
    name: &str,
    values: &serde_json::Map<String, serde_json::Value>,
    style: &str,
    explode: bool,
    allow_reserved: bool,
) {
    let mut serialized = Vec::new();
    for (key, value) in values {
        if value.is_null() {
            continue;
        }
        if style == "form" && explode {
            pairs.push(format!("{}={}", percent_encode(key), encode_query_value(&primitive_to_string(value), allow_reserved)));
        } else {
            serialized.push(key.clone());
            serialized.push(primitive_to_string(value));
        }
    }
    if !serialized.is_empty() {
        pairs.push(format!("{}={}", percent_encode(name), encode_query_value(&serialized.join(","), allow_reserved)));
    }
}

fn append_deep_object_parameter(
    pairs: &mut Vec<String>,
    name: &str,
    values: &serde_json::Map<String, serde_json::Value>,
    allow_reserved: bool,
) {
    for (key, value) in values {
        if !value.is_null() {
            pairs.push(format!("{}={}", percent_encode(&format!("{}[{}]", name, key)), encode_query_value(&primitive_to_string(value), allow_reserved)));
        }
    }
}

fn encode_query_value(value: &str, allow_reserved: bool) -> String {
    let mut encoded = percent_encode(value);
    if !allow_reserved {
        return encoded;
    }
    for (escaped, reserved) in [
        ("%3A", ":"), ("%2F", "/"), ("%3F", "?"), ("%23", "#"),
        ("%5B", "["), ("%5D", "]"), ("%40", "@"), ("%21", "!"),
        ("%24", "$"), ("%26", "&"), ("%27", "'"), ("%28", "("),
        ("%29", ")"), ("%2A", "*"), ("%2B", "+"), ("%2C", ","),
        ("%3B", ";"), ("%3D", "="),
    ] {
        encoded = encoded.replace(escaped, reserved);
    }
    encoded
}

fn primitive_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        other => other.to_string(),
    }
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{:02X}", byte).chars().collect(),
        })
        .collect()
}
