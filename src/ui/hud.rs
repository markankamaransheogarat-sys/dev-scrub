use std::collections::HashMap;
use crate::scanner::{ArtifactType, ScanReport};
use crate::cleaner::CleanResult;

pub struct TerminalHud;

impl TerminalHud {
    pub fn format_scan_report(report: &ScanReport) -> String {
        let mut out = String::new();

        out.push_str("\n┌─────────────────────────────────────────────────────────────────────────────┐\n");
        out.push_str(&format!("│ 🧹 DEV-SCRUB: DISK HYGIENE REPORT FOR {:<37} │\n", report.root_path.display()));
        out.push_str("└─────────────────────────────────────────────────────────────────────────────┘\n\n");

        if report.items.is_empty() {
            out.push_str("  ✨ Disk is clean! No heavy build artifacts detected.\n");
            return out;
        }

        // 1. Group by Artifact Type
        let mut type_totals: HashMap<ArtifactType, (u64, usize)> = HashMap::new();
        for item in &report.items {
            let entry = type_totals.entry(item.artifact_type).or_insert((0, 0));
            entry.0 += item.size_bytes;
            entry.1 += 1;
        }

        out.push_str("📦 [RECLAIMABLE ARTIFACT BREAKDOWN]\n");
        for (art_type, (bytes, count)) in type_totals {
            let mb = bytes as f64 / (1024.0 * 1024.0);
            let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let size_str = if gb >= 1.0 {
                format!("{gb:.2} GB")
            } else {
                format!("{mb:.1} MB")
            };
            out.push_str(&format!("  • {:<24} -> {:>9} across {:>3} project(s)\n", art_type.name(), size_str, count));
        }
        out.push('\n');

        // 2. Individual Artifacts Table (top 15 largest)
        let mut sorted_items = report.items.clone();
        sorted_items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

        out.push_str("🔍 [HEAVIEST ARTIFACT DIRECTORIES]\n");
        out.push_str("  -----------------------------------------------------------------------------\n");
        out.push_str("  PROJECT              TYPE            SIZE       INACTIVITY   PATH\n");
        out.push_str("  -----------------------------------------------------------------------------\n");

        for item in sorted_items.iter().take(15) {
            let size_str = if item.size_gb() >= 1.0 {
                format!("{:.2} GB", item.size_gb())
            } else {
                format!("{:.1} MB", item.size_mb())
            };

            let inact_str = match item.days_since_modified {
                Some(d) if d > 30 => format!("{d}d ago ⚠️"),
                Some(d) => format!("{d}d ago"),
                None => "n/a".to_string(),
            };

            let trunc_path = item.path.display().to_string();
            let short_path = if trunc_path.len() > 30 {
                format!("...{}", &trunc_path[trunc_path.len() - 27..])
            } else {
                trunc_path
            };

            out.push_str(&format!(
                "  {:<20} {:<15} {:>9}  {:<12} {}\n",
                Self::truncate(&item.project_name, 20),
                Self::truncate(item.artifact_type.name(), 15),
                size_str,
                inact_str,
                short_path
            ));
        }
        out.push('\n');

        // 3. Summary
        out.push_str("===============================================================================\n");
        out.push_str(&format!(
            "📊 TOTAL RECLAIMABLE SPACE: {:.2} GB ({:.1} MB) across {} directories\n",
            report.total_gb(),
            report.total_mb(),
            report.items.len()
        ));
        out.push_str(&format!("⚡ Scan duration: {:.2} ms (scanned {} files)\n", report.duration_ms, report.total_files));
        out.push_str("===============================================================================\n");

        out
    }

    pub fn format_clean_result(result: &CleanResult) -> String {
        let mut out = String::new();
        out.push_str("\n===============================================================================\n");
        if result.dry_run {
            out.push_str("🧪 DEV-SCRUB DRY-RUN SIMULATION (NO FILES WERE DELETED)\n");
            out.push_str(&format!(
                "  Potential Space Recoverable: {:.2} GB ({:.1} MB) across {} directories\n",
                result.reclaimed_gb(),
                result.reclaimed_mb(),
                result.cleaned_items
            ));
            out.push_str("  Run with `dev-scrub clean --all --force` to execute actual deletion.\n");
        } else {
            out.push_str("🧹 DEV-SCRUB VACUUM COMPLETE!\n");
            out.push_str(&format!(
                "  Reclaimed Space:   {:.2} GB ({:.1} MB)\n",
                result.reclaimed_gb(),
                result.reclaimed_mb()
            ));
            out.push_str(&format!("  Directories Cleaned: {}\n", result.cleaned_items));
            if result.failed_items > 0 {
                out.push_str(&format!("  Failed Cleanups:     {}\n", result.failed_items));
            }
        }
        out.push_str("===============================================================================\n");
        out
    }

    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() > max_len {
            format!("{}...", &s[..max_len - 3])
        } else {
            s.to_string()
        }
    }
}
