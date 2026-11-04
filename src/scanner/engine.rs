use std::path::{Path, PathBuf};
use std::time::Instant;
use super::artifact::{ArtifactItem, ArtifactType};
use super::git_meta::GitInspector;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanReport {
    pub root_path: PathBuf,
    pub items: Vec<ArtifactItem>,
    pub total_bytes: u64,
    pub total_files: usize,
    pub duration_ms: f64,
}

impl ScanReport {
    pub fn total_gb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn filter_by_days(&self, min_days: u64) -> Vec<&ArtifactItem> {
        self.items
            .iter()
            .filter(|item| item.days_since_modified.unwrap_or(0) >= min_days)
            .collect()
    }
}

pub struct DevScanner {
    max_depth: usize,
}

impl DevScanner {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    pub fn with_defaults() -> Self {
        Self::new(8)
    }

    pub fn scan_path(&self, root: &Path) -> ScanReport {
        let start = Instant::now();
        let mut items = Vec::new();

        self.walk_recursive(root, 0, &mut items);

        let total_bytes = items.iter().map(|it| it.size_bytes).sum();
        let total_files = items.iter().map(|it| it.file_count).sum();
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        ScanReport {
            root_path: root.to_path_buf(),
            items,
            total_bytes,
            total_files,
            duration_ms,
        }
    }

    fn walk_recursive(&self, dir: &Path, depth: usize, items: &mut Vec<ArtifactItem>) {
        if depth > self.max_depth {
            return;
        }

        let read_dir = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return, // Permission denied or inaccessible, skip gracefully
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };

            // Optimization 1: Skip .git directories entirely!
            if file_name == ".git" {
                continue;
            }

            // Optimization 2: Check if directory is a known heavy build artifact
            if let Some(artifact_type) = ArtifactType::matches_dir(file_name) {
                // Verified artifact root! Measure size without descending deeper
                let (size, count, latest_mtime) = self.measure_dir(&path);
                if size > 0 {
                    let parent = path.parent().unwrap_or(dir);
                    let project_name = parent
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let (_, days_since) = GitInspector::inspect_project(parent);

                    items.push(ArtifactItem {
                        path,
                        artifact_type,
                        size_bytes: size,
                        file_count: count,
                        last_modified: latest_mtime,
                        project_name,
                        days_since_modified: days_since,
                    });
                }
                // PRUNE: Do not recurse into target/ or node_modules/ subdirectories!
                continue;
            }

            // Recurse into project subdirectories
            self.walk_recursive(&path, depth + 1, items);
        }
    }

    fn measure_dir(&self, dir: &Path) -> (u64, usize, Option<std::time::SystemTime>) {
        let mut total_size = 0u64;
        let mut total_count = 0usize;
        let mut latest_mtime = None;

        let mut stack = vec![dir.to_path_buf()];

        while let Some(current) = stack.pop() {
            if let Ok(rd) = std::fs::read_dir(&current) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else if let Ok(meta) = p.metadata() {
                        total_size += meta.len();
                        total_count += 1;
                        if let Ok(mtime) = meta.modified() {
                            latest_mtime = Some(latest_mtime.map_or(mtime, |prev: std::time::SystemTime| prev.max(mtime)));
                        }
                    }
                }
            }
        }

        (total_size, total_count, latest_mtime)
    }
}
