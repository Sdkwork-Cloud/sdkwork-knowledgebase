use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeObjectKeyPlanner {
    tenant_id: String,
    space_uuid: String,
}

impl KnowledgeObjectKeyPlanner {
    pub fn new(
        tenant_id: impl Into<String>,
        space_uuid: impl Into<String>,
    ) -> Result<Self, ObjectKeyPlanError> {
        let tenant_id = safe_segment(&tenant_id.into())?;
        let space_uuid = safe_segment(&space_uuid.into())?;
        Ok(Self {
            tenant_id,
            space_uuid,
        })
    }

    pub fn root(&self) -> String {
        format!("knowledge/{}/{}/", self.tenant_id, self.space_uuid)
    }

    pub fn llm_wiki_file(&self, logical_path: &str) -> Result<String, ObjectKeyPlanError> {
        let relative_path = safe_relative_path(logical_path)?;
        Ok(format!(
            "knowledge/{}/{}/{}",
            self.tenant_id, self.space_uuid, relative_path
        ))
    }

    pub fn raw_source_original(
        &self,
        source_uuid: &str,
        display_file_name: &str,
    ) -> Result<String, ObjectKeyPlanError> {
        let source_uuid = safe_segment(source_uuid)?;
        let file_name = safe_file_name(display_file_name)?;
        Ok(format!(
            "knowledge/{}/{}/sources/raw/{}/original/{}",
            self.tenant_id, self.space_uuid, source_uuid, file_name
        ))
    }
}

pub fn safe_file_name(display_name: &str) -> Result<String, ObjectKeyPlanError> {
    let last_segment = display_name
        .replace('\\', "/")
        .split('/')
        .rfind(|segment| !segment.is_empty() && *segment != "." && *segment != "..")
        .unwrap_or("")
        .to_string();

    let mut output = String::new();
    let mut last_was_dash = false;
    for ch in last_segment.chars() {
        let normalized = if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' {
            ch
        } else {
            '-'
        };

        if normalized == '-' {
            if !last_was_dash {
                output.push(normalized);
                last_was_dash = true;
            }
        } else {
            output.push(normalized);
            last_was_dash = false;
        }
    }

    while output.contains("-.") {
        output = output.replace("-.", ".");
    }

    let output = output.trim_matches(['-', '.']).to_string();
    if output.is_empty() {
        return Err(ObjectKeyPlanError::UnsafePath(display_name.to_string()));
    }
    Ok(output)
}

fn safe_relative_path(path: &str) -> Result<String, ObjectKeyPlanError> {
    if path.starts_with('/') || path.starts_with('\\') || path.contains(':') {
        return Err(ObjectKeyPlanError::UnsafePath(path.to_string()));
    }

    let normalized = path.replace('\\', "/");
    let mut segments = Vec::new();
    for segment in normalized.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(ObjectKeyPlanError::UnsafePath(path.to_string()));
        }
        segments.push(safe_segment(segment)?);
    }

    Ok(segments.join("/"))
}

fn safe_segment(segment: &str) -> Result<String, ObjectKeyPlanError> {
    if segment.is_empty()
        || segment == "."
        || segment == ".."
        || segment.contains('/')
        || segment.contains('\\')
        || segment.contains(':')
    {
        return Err(ObjectKeyPlanError::UnsafePath(segment.to_string()));
    }

    if !segment
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
    {
        return Err(ObjectKeyPlanError::UnsafePath(segment.to_string()));
    }

    Ok(segment.to_string())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ObjectKeyPlanError {
    #[error("unsafe knowledge object path: {0}")]
    UnsafePath(String),
}
