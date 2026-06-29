use sdkwork_intelligence_knowledgebase_service::okf::ImportOkfBundleFile;
use std::path::{Path, PathBuf};

pub fn stackoverflow_bundle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../external/knowledge-catalog/okf/bundles/stackoverflow")
}

pub fn discover_bundle_files_from_directory(
    root: &Path,
) -> Result<Vec<ImportOkfBundleFile>, std::io::Error> {
    let mut files = Vec::new();
    discover_bundle_files_from_directory_inner(root, root, &mut files)?;
    files.sort_by(|left, right| left.bundle_relative_path.cmp(&right.bundle_relative_path));
    Ok(files)
}

fn discover_bundle_files_from_directory_inner(
    root: &Path,
    current: &Path,
    files: &mut Vec<ImportOkfBundleFile>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            discover_bundle_files_from_directory_inner(root, &path, files)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let bundle_relative_path = path
            .strip_prefix(root)
            .map_err(|error| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string())
            })?
            .to_string_lossy()
            .replace('\\', "/");
        let markdown = std::fs::read_to_string(&path)?;
        files.push(ImportOkfBundleFile {
            bundle_relative_path,
            markdown,
        });
    }
    Ok(())
}
