use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum ArtifactType {
    RustTarget,
    NodeModules,
    PythonVenv,
    GradleBuild,
    GoBin,
    GeneralCache,
}

impl ArtifactType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::RustTarget => "Rust (target/)",
            Self::NodeModules => "Node.js (node_modules/)",
            Self::PythonVenv => "Python (.venv/cache)",
            Self::GradleBuild => "Gradle/Java (build/)",
            Self::GoBin => "Go (bin/pkg)",
            Self::GeneralCache => "Build Cache (.cache/)",
        }
    }

    pub fn matches_dir(dir_name: &str) -> Option<Self> {
        match dir_name {
            "target" => Some(Self::RustTarget),
            "node_modules" | ".next" | ".turbo" => Some(Self::NodeModules),
            ".venv" | "venv" | "__pycache__" | ".pytest_cache" | ".mypy_cache" | ".ruff_cache" => {
                Some(Self::PythonVenv)
            }
            ".gradle" => Some(Self::GradleBuild),
            ".cache" => Some(Self::GeneralCache),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArtifactItem {
    pub path: PathBuf,
    pub artifact_type: ArtifactType,
    pub size_bytes: u64,
    pub file_count: usize,
    pub last_modified: Option<SystemTime>,
    pub project_name: String,
    pub days_since_modified: Option<u64>,
}

impl ArtifactItem {
    pub fn size_mb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}
