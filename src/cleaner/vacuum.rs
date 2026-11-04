use std::path::Path;
use crate::scanner::{ArtifactItem, ArtifactType};

#[derive(Debug, Clone)]
pub struct CleanOptions {
    pub dry_run: bool,
    pub min_days_inactive: Option<u64>,
    pub filter_types: Option<Vec<ArtifactType>>,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            dry_run: true, // Default to dry-run for absolute safety
            min_days_inactive: None,
            filter_types: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CleanResult {
    pub dry_run: bool,
    pub cleaned_items: usize,
    pub reclaimed_bytes: u64,
    pub failed_items: usize,
    pub errors: Vec<(String, String)>,
}

impl CleanResult {
    pub fn reclaimed_mb(&self) -> f64 {
        self.reclaimed_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn reclaimed_gb(&self) -> f64 {
        self.reclaimed_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

pub struct VacuumEngine;

impl VacuumEngine {
    /// Validates safety invariants before any deletion
    pub fn is_safe_to_delete(path: &Path) -> Result<(), &'static str> {
        let canonical = path.canonicalize().map_err(|_| "Path does not exist")?;

        // 1. Never delete root or direct top-level mounts
        if canonical.parent().is_none() || canonical.as_os_str() == "/" || canonical.as_os_str() == "/root" || canonical.as_os_str() == "/home" {
            return Err("Safety violation: Refusing to delete system or home root");
        }

        // 2. Directory name must be an artifact name
        let dir_name = match canonical.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => return Err("Safety violation: Inaccessible directory name"),
        };

        if ArtifactType::matches_dir(dir_name).is_none() {
            return Err("Safety violation: Directory name is not a recognized build artifact");
        }

        // 3. Never delete git or source dirs
        if dir_name == ".git" || dir_name == "src" {
            return Err("Safety violation: Refusing to delete source or git directory");
        }

        Ok(())
    }

    pub fn execute(items: &[ArtifactItem], options: &CleanOptions) -> CleanResult {
        let mut cleaned_items = 0;
        let mut reclaimed_bytes = 0u64;
        let mut failed_items = 0;
        let mut errors = Vec::new();

        for item in items {
            // Check type filter
            if let Some(ref types) = options.filter_types {
                if !types.contains(&item.artifact_type) {
                    continue;
                }
            }

            // Check inactivity filter
            if let Some(min_days) = options.min_days_inactive {
                let days = item.days_since_modified.unwrap_or(0);
                if days < min_days {
                    continue;
                }
            }

            // Verify safety invariants
            if let Err(e) = Self::is_safe_to_delete(&item.path) {
                errors.push((item.path.display().to_string(), e.to_string()));
                failed_items += 1;
                continue;
            }

            if options.dry_run {
                cleaned_items += 1;
                reclaimed_bytes += item.size_bytes;
            } else {
                match std::fs::remove_dir_all(&item.path) {
                    Ok(_) => {
                        cleaned_items += 1;
                        reclaimed_bytes += item.size_bytes;
                    }
                    Err(e) => {
                        failed_items += 1;
                        errors.push((item.path.display().to_string(), e.to_string()));
                    }
                }
            }
        }

        CleanResult {
            dry_run: options.dry_run,
            cleaned_items,
            reclaimed_bytes,
            failed_items,
            errors,
        }
    }
}
