use std::path::Path;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum GitActivity {
    Active,  // Modified or committed < 7 days ago
    Stale,   // 7 - 30 days ago
    Dormant, // > 30 days ago (Prime cleanup target!)
    NonGit,
}

impl GitActivity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Active => "Active (<7d)",
            Self::Stale => "Stale (7-30d)",
            Self::Dormant => "DORMANT (>30d ⚠️)",
            Self::NonGit => "Non-Git",
        }
    }
}

pub struct GitInspector;

impl GitInspector {
    /// Detects git activity by inspecting the project's `.git` directory
    pub fn inspect_project(project_root: &Path) -> (GitActivity, Option<u64>) {
        let git_dir = project_root.join(".git");
        if !git_dir.exists() {
            return (GitActivity::NonGit, None);
        }

        // Try reading latest ref modification from .git/logs/HEAD or .git/HEAD
        let check_files = [
            git_dir.join("logs").join("HEAD"),
            git_dir.join("FETCH_HEAD"),
            git_dir.join("HEAD"),
        ];

        let mut latest_mtime: Option<SystemTime> = None;

        for f in &check_files {
            if let Ok(meta) = std::fs::metadata(f) {
                if let Ok(mtime) = meta.modified() {
                    latest_mtime = Some(latest_mtime.map_or(mtime, |prev| prev.max(mtime)));
                }
            }
        }

        if let Some(mtime) = latest_mtime {
            let elapsed = SystemTime::now()
                .duration_since(mtime)
                .unwrap_or(Duration::from_secs(0));

            let days = elapsed.as_secs() / 86400;

            let activity = if days < 7 {
                GitActivity::Active
            } else if days <= 30 {
                GitActivity::Stale
            } else {
                GitActivity::Dormant
            };

            (activity, Some(days))
        } else {
            (GitActivity::Stale, None)
        }
    }
}
