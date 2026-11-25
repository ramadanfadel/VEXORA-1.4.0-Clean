//! VEXORA HyperScan Engine – adapter module
//! Wraps czkawka-core duplicate finding with result mapping.
//! Currently, the engine runs via the dedicated command; the adapter structure
//! is defined for future integration.

use czkawka_core::common::model::{HashType, CheckingMethod};
use czkawka_core::common::traits::Search;
use czkawka_core::tools::duplicate::{DuplicateFinder, DuplicateFinderParameters};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Engine selection enum used by the UI to choose which duplicate finder to run.
// ---------------------------------------------------------------------------
/// Enumerate which duplicate‑finder the user wants to run.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ScanEngine {
    /// Original VEXORA size‑+‑hash finder (deterministic, well‑tested).
    Classic,
    /// Experimental engine powered by czkawka‑core (requires the binary dependency).
    HyperScan,
}

/// Default engine – Classic – so we don’t break existing behaviour.
pub const DEFAULT_ENINE: ScanEngine = ScanEngine::Classic;

// ---------------------------------------------------------------------------
// Result carrier returned to the UI.
// ---------------------------------------------------------------------------
/// Result of a duplicate‑scan operation, carrying information about which
/// engine produced the results so the UI can display it.
#[derive(Clone, Debug, Serialize)]
pub struct ScanResult {
    /// The engine that produced these results.
    pub engine: ScanEngine,
    /// The underlying duplicate groups (as strings for UI display).
    pub groups: Vec<String>,
    /// Total reclaimed space if all but one copy in each group were deleted.
    pub reclaimable: u64,
    /// Total number of files scanned.
    pub scanned_files: u64,
    /// Number of errors encountered (e.g. access‑denied).
    pub errors: u64,
}

// ---------------------------------------------------------------------------
// Convert a ScanEngine to a human‑readable label for the UI.
// ---------------------------------------------------------------------------
pub fn engine_label(engine: ScanEngine) -> &'static str {
    match engine {
        ScanEngine::Classic => "VEXORA Classic",
        ScanEngine::HyperScan => "VEXORA HyperScan (Experimental)",
    }
}

// ---------------------------------------------------------------------------
// Run a duplicate scan using the specified engine.
// ---------------------------------------------------------------------------
/// Run a duplicate scan using the specified engine.
/// * `engine` – which algorithm to use.
/// * `root` – root directory to scan.
/// * `min_size` – minimum file size (bytes) to include.
pub fn scan(engine: ScanEngine, _root: &str, min_size: u64) -> ScanResult {
    match engine {
        ScanEngine::Classic => {
            // Use the original VEXORA finder via tools::find_duplicates.
            // (In a real integration this would call `tools::find_duplicates`.)
            ScanResult {
                engine: ScanEngine::Classic,
                groups: Vec::new(),
                reclaimable: 0,
                scanned_files: 0,
                errors: 0,
            }
        }
        ScanEngine::HyperScan => {
            // Use czkawka‑core DuplicateFinder.
            // We create parameters with Blake3 hash and Hash checking method.
            let params = DuplicateFinderParameters::new(
                CheckingMethod::Hash,
                HashType::Blake3,
                false, // case_sensitive_name_comparison
                0,     // minimal_prehash_cache_file_size
                min_size, // prehash_cache_min_file_size (use min_size as threshold)
                false, // use_prehash_cache
            );

            // According to czkawka-core v12 API, new returns the finder directly.
            use std::sync::Arc;
            let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let mut finder = DuplicateFinder::new(params);

            // Run the search with a stop flag and no progress sender.
            // The stop_flag allows cancellation; progress_sender is None for now.
            let _result = finder.search(&stop_flag, None);

            // Map the czkawka-core result to our ScanResult.
            // For now return placeholder; real population depends on result type.
            ScanResult {
                engine: ScanEngine::HyperScan,
                groups: Vec::new(),
                reclaimable: 0,
                scanned_files: 0,
                errors: 0,
            }
        }
    }
}