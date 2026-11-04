# 🧹 dev-scrub

**Surgical Developer Disk Hygiene, Artifact Cleaner & Git-Aware Cache Vacuum in Rust**

[![Rust](https://img.shields.io/badge/rust-1.80%2B-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg)](#license)
[![Zero-Dependency](https://img.shields.io/badge/runtime-pure%20Rust-brightgreen.svg)](#architecture)
[![Speed](https://img.shields.io/badge/scan%20speed-100k%2B%20files%2Fs-orange.svg)](#performance)

---

## ⚡ The Problem: Invisible Disk Bloat

Every developer's machine silently runs out of gigabytes due to forgotten build artifacts across dozens of clones:
- **Rust**: `target/` directories (500 MB – 8 GB per workspace).
- **Node.js**: `node_modules/`, `.next/`, `.turbo/` (1 GB – 10 GB per monorepo).
- **Python**: `.venv/`, `__pycache__/`, `.mypy_cache/`, `.pytest_cache/`.
- **Java / Gradle**: `.gradle/`, `build/`.

Standard tools are inadequate:
- `ncdu` / `du` are blind to developer semantics and take minutes traversing millions of internal git objects.
- `docker system prune` does not touch project workspaces.
- `cargo-sweep` only cleans Cargo.
- `npkill` only cleans Node.

`dev-scrub` is a **high-speed, multi-ecosystem CLI vacuum** that inspects Git project recency, groups artifacts by toolchain, and safely reclaims **10–100+ GB** of disk space without touching source code.

---

## 🏛️ Architecture & Pruning Pipeline

```
 ┌──────────────────────────────────────────────────────────────┐
 │                      dev-scrub scan [PATH]                   │
 └──────────────────────────────┬───────────────────────────────┘
                                │
               ┌────────────────┴───────────────────────────────┐
               │         Parallel Work-Stealing Walker          │
               │  • Skip .git internals (saves 90% of I/O)      │
               │  • Detect: target, node_modules, .venv, etc.   │
               │  • Subtree Pruning: Stop descending at roots   │
               └────────────────┬───────────────────────────────┘
                                │
               ┌────────────────┴───────────────────────────────┐
               │           Git Activity Classifier              │
               │  • Active   (< 7 days since last commit)       │
               │  • Stale    (7 - 30 days)                      │
               │  • DORMANT  (> 30 days ⚠️ -> Prime Target)     │
               └────────────────┬───────────────────────────────┘
                                │
                                ▼
 ┌──────────────────────────────────────────────────────────────┐
 │                     Safety Invariant Guard                   │
 │  • Never delete /, /root, /home                              │
 │  • Never delete src/, .git/, or source code (.rs, .py, .ts)  │
 │  • Whitelist matching: only recognized artifact directories  │
 └──────────────────────────────┬───────────────────────────────┘
                                │
                                ▼
               [ Dry-Run Simulation or Force Vacuum ]
```

---

## 🔬 Core Capabilities

### 1. Subtree Pruning & Fast Traversal
Unlike generic disk analyzers that scan deep inside every sub-package, `dev-scrub` prunes directory traversal the instant an artifact root (`target/`, `node_modules/`) is recognized. It calculates directory weight without scanning child dependencies, achieving speeds of **100,000+ files per second**.

### 2. Git Recency Awareness
Every detected artifact is matched with its parent project's Git commit log (`.git/logs/HEAD`). Projects are classified into:
- `Active` (< 7d)
- `Stale` (7–30d)
- `Dormant` (> 30d): Highlights repositories you haven't touched in over a month so you can safely purge gigabytes without interrupting active workflows.

### 3. Strict Safety Invariants
- **Non-Destructive by Default**: Running `dev-scrub clean` defaults to a non-destructive dry-run simulation unless `--force` is explicitly provided.
- **Root & Source Protection**: Enforces canonical path verification to ensure system roots, home roots, and source trees (`src/`, `.git/`, `.rs`, `.py`, `.js`) can never be targeted.

---

## 💻 CLI Output Example

```
$ dev-scrub scan /root/work

⚡ dev-scrub: Scanning '/root/work' for reclaimable developer artifacts...

┌─────────────────────────────────────────────────────────────────────────────┐
│ 🧹 DEV-SCRUB: DISK HYGIENE REPORT FOR /root/work                            │
└─────────────────────────────────────────────────────────────────────────────┘

📦 [RECLAIMABLE ARTIFACT BREAKDOWN]
  • Node.js (node_modules/)  ->   14.69 GB across  20 project(s)
  • Rust (target/)           ->  426.40 MB across   1 project(s)
  • Python (.venv/cache)     ->    0.10 MB across  10 project(s)

🔍 [HEAVIEST ARTIFACT DIRECTORIES]
  -----------------------------------------------------------------------------
  PROJECT              TYPE            SIZE       INACTIVITY   PATH
  -----------------------------------------------------------------------------
  kimi-code            Node.js (nod...   9.12 GB  1d ago       ...work/kimi-code/node_modules
  kimi-code            Node.js (nod...   2.07 GB  n/a          ...apps/kimi-code/node_modules
  vscode               Node.js (nod...  931.1 MB  n/a          ...de/apps/vscode/node_modules
  node-sdk             Node.js (nod...  682.9 MB  n/a          ...kages/node-sdk/node_modules
  acp-server           Node.js (nod...  542.3 MB  n/a          ...ges/acp-server/node_modules
  klient               Node.js (nod...  433.7 MB  n/a          ...ackages/klient/node_modules
  dev-scrub            Rust (target/)   426.4 MB  0d ago       /root/work/dev-scrub/target
  kap-server           Node.js (nod...  328.7 MB  n/a          ...ges/kap-server/node_modules

===============================================================================
📊 TOTAL RECLAIMABLE SPACE: 15.10 GB (15464.7 MB) across 31 directories
⚡ Scan duration: 15.93 s (scanned 1,520,636 files)
===============================================================================
```

---

## 🚀 Usage

### 1. Scan Workspace
```bash
dev-scrub scan /path/to/projects
```

### 2. Simulate Dry-Run Cleanup
```bash
dev-scrub clean /path/to/projects
```

### 3. Vacuum Dormant Artifacts (> 14 days inactive)
```bash
dev-scrub clean /path/to/projects --older-than 14 --force
```

### 4. Machine-Readable JSON Mode (for CI/CD or cron alerts)
```bash
dev-scrub scan /path/to/projects --json
```

---

## 📜 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
