use serde::Serialize;
use crate::session::{Solve, SessionType};

#[derive(Serialize)]
pub struct SessionStats {
    pub solve_count: usize,
    pub best_ms: Option<u32>,
    pub worst_ms: Option<u32>,
    pub average_ms: Option<u32>,
    pub current_ao5_ms: Option<u32>,
    pub current_ao12_ms: Option<u32>,
    pub best_ao5_ms: Option<u32>,
    pub best_ao12_ms: Option<u32>,
    pub mean_tps: Option<f64>,
    pub session_type: String,
    /// WCA: milliseconds remaining in the 1-hour window. None if not WCA or no solves yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wca_remaining_ms: Option<i64>,
    /// WCA: number of solves remaining (out of 5). None if not WCA.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wca_solves_remaining: Option<usize>,
    /// WCA: true if session has 5 solves (complete).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub wca_complete: bool,
}

#[derive(Serialize)]
pub struct SolveListEntry {
    pub id: String,
    pub index: usize,
    pub time_ms: u32,
    pub turns: usize,
    pub tps: f64,
    pub is_best: bool,
    pub penalty: Option<String>,
}

/// Trimmed mean of `n` values with DNF support (WCA rules).
/// - `None` = DNF (treated as +infinity for sorting)
/// - 2+ DNFs in window → entire average is DNF (returns None)
/// - 1 DNF → trimmed as worst, average computed from rest
/// - Result rounded to nearest centisecond (10ms) per WCA rule 9f6
pub fn compute_ao_n(times: &[Option<u32>], n: usize) -> Option<u32> {
    if times.len() < n || n < 3 {
        return None;
    }
    let window = &times[times.len() - n..];
    trimmed_mean(window)
}

/// Slide a window of size `n` across all times, return the lowest trimmed mean.
/// Skips windows where average is DNF (2+ DNFs).
pub fn compute_best_ao_n(times: &[Option<u32>], n: usize) -> Option<u32> {
    if times.len() < n || n < 3 {
        return None;
    }
    let mut best: Option<u32> = None;
    for start in 0..=(times.len() - n) {
        let window = &times[start..start + n];
        if let Some(avg) = trimmed_mean(window) {
            best = Some(match best {
                Some(b) if avg < b => avg,
                Some(b) => b,
                None => avg,
            });
        }
    }
    best
}

/// WCA trimmed mean: remove best and worst, average the rest.
/// DNFs sort as worst (+infinity). 2+ DNFs → None (DNF average).
/// Result rounded to nearest centisecond (10ms) per WCA rule 9f6.
pub fn trimmed_mean(window: &[Option<u32>]) -> Option<u32> {
    if window.len() < 3 {
        return None;
    }

    let dnf_count = window.iter().filter(|t| t.is_none()).count();
    if dnf_count >= 2 {
        return None; // 2+ DNFs → entire average is DNF
    }

    // Sort: Some values ascending, None (DNF) at end
    let mut sorted: Vec<Option<u32>> = window.to_vec();
    sorted.sort_by(|a, b| match (a, b) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    // Remove best (first) and worst (last)
    let trimmed = &sorted[1..sorted.len() - 1];

    // All remaining must be Some (at most 1 DNF which sorted to end and got trimmed)
    let n = trimmed.len() as u64;
    let sum: u64 = trimmed.iter().map(|t| t.unwrap() as u64).sum();

    // Round to nearest centisecond (10ms) per WCA rule 9f6
    let avg_tenths = sum * 10 / n; // avg in 0.1ms units (extra precision digit)
    let centiseconds = (avg_tenths + 50) / 100; // round to nearest centisecond
    Some((centiseconds * 10) as u32)
}

pub fn compute_tps(solve: &Solve) -> f64 {
    if solve.time == 0 {
        return 0.0;
    }
    consolidated_move_count(&solve.moves) as f64 / (solve.time as f64 / 1000.0)
}

/// Count moves after consolidating consecutive identical quarter turns into doubles.
/// e.g. ["D", "D", "R'"] → 2 (D2 + R')
pub fn consolidated_move_count(moves: &[String]) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < moves.len() {
        count += 1;
        // If current and next are identical quarter turns, they consolidate to one double
        if !moves[i].ends_with('2') && i + 1 < moves.len() && moves[i] == moves[i + 1] {
            i += 2;
        } else {
            i += 1;
        }
    }
    count
}

pub fn compute_session_stats(solves: &[Solve], session_type: SessionType, first_solve_at: Option<i64>, now_ms: i64) -> SessionStats {
    // All valid solves sorted by date (deleted solves already filtered at storage level,
    // but also filter here for in-memory consistency after soft-delete)
    let mut all_solves: Vec<&Solve> = solves.iter()
        .filter(|s| s.is_valid && s.deleted_at.is_none())
        .collect();
    all_solves.sort_by_key(|s| s.date);

    // Non-DNF solves for simple stats (best, worst, average, TPS)
    let non_dnf: Vec<&Solve> = all_solves.iter()
        .filter(|s| s.penalty.is_none())
        .copied()
        .collect();
    let times: Vec<u32> = non_dnf.iter().map(|s| s.time).collect();

    // Ao times: None = DNF, Some(time) = normal — includes DNFs per WCA rules
    let ao_times: Vec<Option<u32>> = all_solves.iter()
        .map(|s| if s.penalty.as_deref() == Some("DNF") { None } else { Some(s.time) })
        .collect();

    let best_ms = times.iter().copied().min();
    let worst_ms = times.iter().copied().max();
    let average_ms = if times.is_empty() {
        None
    } else {
        let sum: u64 = times.iter().map(|&t| t as u64).sum();
        Some((sum / times.len() as u64) as u32)
    };

    let current_ao5_ms = compute_ao_n(&ao_times, 5);
    let current_ao12_ms = compute_ao_n(&ao_times, 12);
    let best_ao5_ms = compute_best_ao_n(&ao_times, 5);
    let best_ao12_ms = compute_best_ao_n(&ao_times, 12);

    let mean_tps = if non_dnf.is_empty() {
        None
    } else {
        let tps_sum: f64 = non_dnf.iter().map(|s| compute_tps(s)).sum();
        Some(tps_sum / non_dnf.len() as f64)
    };

    let session_type_str = match session_type {
        SessionType::Free => "Free".to_string(),
        SessionType::WCA => "WCA".to_string(),
    };

    let (wca_remaining_ms, wca_solves_remaining, wca_complete) = match session_type {
        SessionType::WCA => {
            let complete = all_solves.len() >= 5;
            let remaining = if complete {
                None // Don't show countdown when session is done
            } else {
                first_solve_at.map(|first| {
                    let one_hour_ms: i64 = 3600 * 1000;
                    (one_hour_ms - (now_ms - first)).max(0)
                })
            };
            let solves_left = Some(5_usize.saturating_sub(all_solves.len()));
            (remaining, solves_left, complete)
        }
        _ => (None, None, false),
    };

    SessionStats {
        solve_count: all_solves.len(), // Total solves including DNF
        best_ms,
        worst_ms,
        average_ms,
        current_ao5_ms,
        current_ao12_ms,
        best_ao5_ms,
        best_ao12_ms,
        mean_tps,
        session_type: session_type_str,
        wca_remaining_ms,
        wca_solves_remaining,
        wca_complete,
    }
}

pub fn compute_solve_list(solves: &[Solve]) -> Vec<SolveListEntry> {
    // Sort by date to ensure chronological order, exclude soft-deleted
    let mut sorted: Vec<&Solve> = solves.iter().filter(|s| s.is_valid && s.deleted_at.is_none()).collect();
    sorted.sort_by_key(|s| s.date);

    // Best time only among non-DNF solves
    let best_time = sorted.iter()
        .filter(|s| s.penalty.is_none())
        .map(|s| s.time)
        .min();

    let mut entries: Vec<SolveListEntry> = sorted
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let tps = compute_tps(s);
            let is_dnf = s.penalty.as_deref() == Some("DNF");
            SolveListEntry {
                id: s.id.clone(),
                index: i + 1,
                time_ms: s.time,
                turns: consolidated_move_count(&s.moves),
                tps,
                is_best: !is_dnf && best_time == Some(s.time),
                penalty: s.penalty.clone(),
            }
        })
        .collect();

    // Reverse for newest-first
    entries.reverse();
    entries
}

