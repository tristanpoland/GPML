use std::path::Path;
use anyhow::{anyhow, Result};
use rust_embed::RustEmbed;

/// Embedded GPML file assets for bundled builds
#[derive(RustEmbed)]
#[folder = "../../"]
#[include = "*.gpml"]
#[include = "examples/**/*.gpml"]
#[exclude = "target/**/*"]
#[exclude = ".git/**/*"]
#[exclude = ".*"]
#[cfg(feature = "bundle")]
pub struct BundledGPMLAssets;

#[cfg(feature = "bundle")]
impl BundledGPMLAssets {
    /// Load a GPML file from the bundled assets
    pub fn load_gpml_file(path: &str) -> Result<String> {
        // Normalize the path to use forward slashes for RustEmbed
        let normalized_path = path.replace('\\', "/");

        // Try the path as-is first
        if let Some(file) = Self::get(&normalized_path) {
            return std::str::from_utf8(&file.data)
                .map(|s| s.to_string())
                .map_err(|e| anyhow!("Failed to decode GPML file as UTF-8: {}", e));
        }

        // If that didn't work, try with various path prefixes since the bundle
        // is created from the project root (../../ from crates/gpml)
        let alternative_paths = vec![
            normalized_path.clone(),
            format!("./{}", normalized_path),
            format!("../{}", normalized_path),
            format!("../../{}", normalized_path),
        ];

        for alt_path in alternative_paths {
            if let Some(file) = Self::get(&alt_path) {
                tracing::debug!("Found GPML file at path: {}", alt_path);
                return std::str::from_utf8(&file.data)
                    .map(|s| s.to_string())
                    .map_err(|e| anyhow!("Failed to decode GPML file as UTF-8: {}", e));
            }
        }

        // List all available files for debugging
        let available_files: Vec<String> = Self::iter().map(|s| s.to_string()).collect();
        tracing::error!("GPML file not found in bundle: {}", path);
        tracing::error!("Available files in bundle: {:?}", available_files);

        Err(anyhow!("GPML file not found in bundle: {} (tried multiple path variations)", path))
    }

    /// Check if a GPML file exists in the bundle
    pub fn gpml_file_exists(path: &str) -> bool {
        let normalized_path = path.replace('\\', "/");

        // Try the path as-is first
        if Self::get(&normalized_path).is_some() {
            return true;
        }

        // Try with various path prefixes
        let alternative_paths = vec![
            normalized_path.clone(),
            format!("./{}", normalized_path),
            format!("../{}", normalized_path),
            format!("../../{}", normalized_path),
        ];

        for alt_path in alternative_paths {
            if Self::get(&alt_path).is_some() {
                return true;
            }
        }

        false
    }

    /// List all bundled GPML files
    pub fn list_gpml_files() -> Vec<String> {
        Self::iter()
            .filter(|path| path.ends_with(".gpml"))
            .map(|path| path.to_string())
            .collect()
    }

    /// Resolve component imports within the bundle
    pub fn resolve_component_path(current_file: &str, import_path: &str) -> Result<String> {
        let current_dir = Path::new(current_file)
            .parent()
            .unwrap_or_else(|| Path::new(""));

        let resolved_path = if import_path.starts_with('/') {
            // Absolute path from bundle root
            import_path.trim_start_matches('/').to_string()
        } else {
            // Relative path from current file
            current_dir.join(import_path)
                .to_string_lossy()
                .replace('\\', "/")
        };

        // Ensure the path has .gpml extension
        let resolved_path = if resolved_path.ends_with(".gpml") {
            resolved_path
        } else {
            format!("{}.gpml", resolved_path)
        };

        if Self::gpml_file_exists(&resolved_path) {
            Ok(resolved_path)
        } else {
            Err(anyhow!("Component file not found in bundle: {}", resolved_path))
        }
    }
}

/// File source that uses either bundled assets OR filesystem, never both
pub struct GPMLFileSource;

impl GPMLFileSource {
    /// Load a file from the appropriate source based on bundle feature
    pub fn load_file(path: &str) -> Result<String> {
        #[cfg(feature = "bundle")]
        {
            // When bundle feature is enabled, ONLY use bundled assets
            BundledGPMLAssets::load_gpml_file(path)
        }
        #[cfg(not(feature = "bundle"))]
        {
            // When bundle feature is disabled, ONLY use filesystem
            std::fs::read_to_string(path)
                .map_err(|e| anyhow!("Failed to read file from filesystem: {} ({})", path, e))
        }
    }

    /// Check if a file exists in the appropriate source
    pub fn file_exists(path: &str) -> bool {
        #[cfg(feature = "bundle")]
        {
            BundledGPMLAssets::gpml_file_exists(path)
        }
        #[cfg(not(feature = "bundle"))]
        {
            Path::new(path).exists()
        }
    }

    /// Resolve component import paths
    pub fn resolve_component_import(current_file: &str, import_path: &str) -> Result<String> {
        #[cfg(feature = "bundle")]
        {
            // When bundle feature is enabled, resolve within bundle
            BundledGPMLAssets::resolve_component_path(current_file, import_path)
        }
        #[cfg(not(feature = "bundle"))]
        {
            // When bundle feature is disabled, resolve on filesystem
            let current_dir = Path::new(current_file)
                .parent()
                .unwrap_or_else(|| Path::new(""));

            let resolved_path = if import_path.starts_with('/') {
                import_path.trim_start_matches('/').to_string()
            } else {
                current_dir.join(import_path)
                    .to_string_lossy()
                    .to_string()
            };

            let resolved_path = if resolved_path.ends_with(".gpml") {
                resolved_path
            } else {
                format!("{}.gpml", resolved_path)
            };

            if Path::new(&resolved_path).exists() {
                Ok(resolved_path)
            } else {
                Err(anyhow!("Component file not found: {}", resolved_path))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpml_file_source() {
        // This test should work with or without bundle feature
        let result = GPMLFileSource::file_exists("non_existent_file.gpml");
        assert_eq!(result, false);
    }

    #[cfg(feature = "bundle")]
    #[test]
    fn test_bundled_assets_list() {
        let files = BundledGPMLAssets::list_gpml_files();
        // Should contain at least the test files we know exist
        assert!(!files.is_empty());
    }
}