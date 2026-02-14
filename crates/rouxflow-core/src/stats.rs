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
fn trimmed_mean(window: &[Option<u32>]) -> Option<u32> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_solve_at(id: &str, time: u32, moves: usize, date: i64) -> Solve {
        Solve {
            id: id.to_string(),
            time,
            moves: (0..moves).map(|i| format!("R{}", i)).collect(),
            date,
            is_valid: true,
            scramble: None,
            timed_moves: None,
            penalty: None,
            deleted_at: None,
        }
    }

    fn make_dnf_at(id: &str, date: i64) -> Solve {
        Solve {
            id: id.to_string(),
            time: 0,
            moves: Vec::new(),
            date,
            is_valid: true,
            scramble: None,
            timed_moves: None,
            penalty: Some("DNF".to_string()),
            deleted_at: None,
        }
    }

    fn make_solve(id: &str, time: u32, moves: usize) -> Solve {
        make_solve_at(id, time, moves, 0)
    }

    // ========== Trimmed Mean Tests ==========

    #[test]
    fn test_trimmed_mean_exact() {
        // avg(2000,3000,4000) = 3000 → exact, no rounding effect
        let window: Vec<Option<u32>> = vec![Some(1000), Some(2000), Some(3000), Some(4000), Some(5000)];
        assert_eq!(trimmed_mean(&window), Some(3000));
    }

    #[test]
    fn test_trimmed_mean_rounds_to_centisecond() {
        // avg(3000,2000,1500) = 6500/3 = 2166.67ms → round to 2170ms (WCA: 2.17s)
        let window: Vec<Option<u32>> = vec![Some(4000), Some(3000), Some(2000), Some(1000), Some(1500)];
        assert_eq!(trimmed_mean(&window), Some(2170));
    }

    #[test]
    fn test_trimmed_mean_rounds_down_at_boundary() {
        // Verify rounding at .5 boundary: 10456.67 → 10460 (10.46s)
        // Window: trim best/worst from [10000, 10200, 10370, 10800, 11000]
        // avg(10200, 10370, 10800) = 31370/3 = 10456.67 → round to 10460
        let window: Vec<Option<u32>> = vec![Some(10000), Some(10200), Some(10370), Some(10800), Some(11000)];
        assert_eq!(trimmed_mean(&window), Some(10460));
    }

    // ========== DNF in Ao Tests ==========

    #[test]
    fn test_ao5_one_dnf_trimmed_as_worst() {
        // User's example: 10.20, DNF, 9.50, 11.05, 10.80
        // Sorted: [9500, 10200, 10800, 11050, DNF]
        // Trim best(9500) and worst(DNF) → avg(10200, 10800, 11050) = 32050/3 = 10683.33
        // Round to centisecond: 10680 (10.68s)
        let times: Vec<Option<u32>> = vec![
            Some(10200), None, Some(9500), Some(11050), Some(10800),
        ];
        assert_eq!(compute_ao_n(&times, 5), Some(10680));
    }

    #[test]
    fn test_ao5_two_dnfs_is_dnf() {
        // 2 DNFs → entire average is DNF
        let times: Vec<Option<u32>> = vec![
            Some(10000), None, Some(9500), None, Some(10800),
        ];
        assert_eq!(compute_ao_n(&times, 5), None);
    }

    #[test]
    fn test_ao5_all_dnf_is_dnf() {
        let times: Vec<Option<u32>> = vec![None, None, None, None, None];
        assert_eq!(compute_ao_n(&times, 5), None);
    }

    #[test]
    fn test_best_ao5_skips_dnf_windows() {
        // 6 solves: [10000, DNF, DNF, 9000, 8000, 7000]
        // Window 0..5: [10000, DNF, DNF, 9000, 8000] → 2 DNFs → DNF (skipped)
        // Window 1..6: [DNF, DNF, 9000, 8000, 7000] → 2 DNFs → DNF (skipped)
        // No valid windows → None
        let times: Vec<Option<u32>> = vec![
            Some(10000), None, None, Some(9000), Some(8000), Some(7000),
        ];
        assert_eq!(compute_best_ao_n(&times, 5), None);

        // Now add a solve to create a valid window at the end
        // [10000, DNF, DNF, 9000, 8000, 7000, 7500]
        // Window 2..7: [DNF, 9000, 8000, 7000, 7500] → 1 DNF → trim 7000,DNF → avg(9000,8000,7500) = 8170
        let mut times2 = times.clone();
        times2.push(Some(7500));
        assert_eq!(compute_best_ao_n(&times2, 5), Some(8170));
    }

    // ========== Ao Computation Tests ==========

    #[test]
    fn test_compute_ao_n_exact() {
        let times: Vec<Option<u32>> = vec![Some(5000), Some(4000), Some(3000), Some(2000), Some(1000)];
        // trim 1000,5000 → avg(4000,3000,2000) = 3000 → exact
        assert_eq!(compute_ao_n(&times, 5), Some(3000));
    }

    #[test]
    fn test_compute_ao_n_insufficient() {
        let times: Vec<Option<u32>> = vec![Some(1000), Some(2000)];
        assert_eq!(compute_ao_n(&times, 5), None);
    }

    #[test]
    fn test_compute_best_ao_n() {
        // 6 solves: [5000, 4000, 3000, 2000, 1000, 1500]
        // Window 0..5: trim→avg(4000,3000,2000) = 3000
        // Window 1..6: trim→avg(3000,2000,1500) = 6500/3 = 2166.67 → round to 2170
        let times: Vec<Option<u32>> = vec![Some(5000), Some(4000), Some(3000), Some(2000), Some(1000), Some(1500)];
        assert_eq!(compute_best_ao_n(&times, 5), Some(2170));
    }

    #[test]
    fn test_compute_tps() {
        let solve = make_solve("1", 10000, 50); // 10s, 50 moves = 5 TPS
        let tps = compute_tps(&solve);
        assert!((tps - 5.0).abs() < 0.001);
    }

    // ========== Session Stats Tests ==========

    #[test]
    fn test_session_stats_empty() {
        let stats = compute_session_stats(&[], SessionType::Free, None, 0);
        assert_eq!(stats.solve_count, 0);
        assert_eq!(stats.best_ms, None);
        assert_eq!(stats.current_ao5_ms, None);
    }

    #[test]
    fn test_session_stats_with_solves() {
        let solves: Vec<Solve> = (1..=5).map(|i| make_solve(&i.to_string(), i * 1000, 20)).collect();
        let stats = compute_session_stats(&solves, SessionType::Free, None, 0);
        assert_eq!(stats.solve_count, 5);
        assert_eq!(stats.best_ms, Some(1000));
        assert_eq!(stats.worst_ms, Some(5000));
        assert_eq!(stats.average_ms, Some(3000));
        assert_eq!(stats.current_ao5_ms, Some(3000)); // trim best+worst → avg(2000,3000,4000)
    }

    #[test]
    fn test_session_stats_with_one_dnf() {
        // 5 solves: 10200, DNF, 9500, 11050, 10800
        let solves = vec![
            make_solve_at("1", 10200, 20, 1),
            make_dnf_at("2", 2),
            make_solve_at("3", 9500, 20, 3),
            make_solve_at("4", 11050, 20, 4),
            make_solve_at("5", 10800, 20, 5),
        ];
        let stats = compute_session_stats(&solves, SessionType::Free, None, 0);
        assert_eq!(stats.solve_count, 5); // DNF counts in total
        assert_eq!(stats.best_ms, Some(9500)); // best excludes DNF
        assert_eq!(stats.current_ao5_ms, Some(10680)); // 1 DNF trimmed as worst
    }

    #[test]
    fn test_session_stats_with_two_dnfs() {
        let solves = vec![
            make_solve_at("1", 10000, 20, 1),
            make_dnf_at("2", 2),
            make_solve_at("3", 9500, 20, 3),
            make_dnf_at("4", 4),
            make_solve_at("5", 10800, 20, 5),
        ];
        let stats = compute_session_stats(&solves, SessionType::Free, None, 0);
        assert_eq!(stats.solve_count, 5);
        assert_eq!(stats.current_ao5_ms, None); // 2 DNFs → DNF average
    }

    #[test]
    fn test_session_stats_deleted_solve_excluded() {
        let mut solves: Vec<Solve> = (1..=5).map(|i| make_solve_at(&i.to_string(), i * 1000, 20, i as i64)).collect();
        // Soft-delete solve #3
        solves[2].deleted_at = Some(99999);
        let stats = compute_session_stats(&solves, SessionType::Free, None, 0);
        assert_eq!(stats.solve_count, 4); // Only 4 solves visible
        assert_eq!(stats.current_ao5_ms, None); // Less than 5 solves → no ao5
    }

    #[test]
    fn test_solve_list_reverse_order() {
        let solves: Vec<Solve> = (1..=3).map(|i| make_solve_at(&i.to_string(), i * 1000, 10, i as i64)).collect();
        let list = compute_solve_list(&solves);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].index, 3); // newest first
        assert_eq!(list[2].index, 1);
        assert!(list[2].is_best); // solve #1 (1000ms) is best
    }

    #[test]
    fn test_current_ao5_differs_from_best_ao5_with_12_solves() {
        // 12 solves with diverse times (in chronological order by date)
        let times_ms = [8500, 9200, 7800, 8100, 9500, 7200, 8800, 9100, 7500, 8300, 9400, 7600];
        let solves: Vec<Solve> = times_ms.iter().enumerate()
            .map(|(i, &t)| make_solve_at(&i.to_string(), t, 20, i as i64 + 1))
            .collect();

        let stats = compute_session_stats(&solves, SessionType::Free, None, 0);

        // Current Ao5 = last 5: [9100, 7500, 8300, 9400, 7600]
        // sorted: [7500, 7600, 8300, 9100, 9400] → trim → avg(7600, 8300, 9100) = 25000/3 = 8333.33
        // WCA round: 8330
        assert_eq!(stats.current_ao5_ms, Some(8330));

        // Best Ao5: window [7200, 8800, 9100, 7500, 8300]
        // sorted: [7200, 7500, 8300, 8800, 9100] → trim → avg(7500, 8300, 8800) = 8200 (exact)
        assert_eq!(stats.best_ao5_ms, Some(8200));

        // They MUST differ
        assert_ne!(stats.current_ao5_ms, stats.best_ao5_ms);
    }

    #[test]
    fn test_stats_sort_by_date_not_insertion_order() {
        // Simulate IndexedDB returning solves in random (non-chronological) order
        let solves = vec![
            make_solve_at("c", 15000, 20, 3),
            make_solve_at("a", 5000, 20, 1),
            make_solve_at("e", 6000, 20, 5),
            make_solve_at("b", 10000, 20, 2),
            make_solve_at("d", 11000, 20, 4),
        ];

        let stats = compute_session_stats(&solves, SessionType::Free, None, 0);
        // After sorting by date: [5000, 10000, 15000, 11000, 6000]
        // Ao5: trim 5000,15000 → avg(10000,11000,6000) = 9000 (exact)
        assert_eq!(stats.current_ao5_ms, Some(9000));

        // Verify solve list is also in date order
        let list = compute_solve_list(&solves);
        assert_eq!(list[0].index, 5); // newest first (date=5, 6000ms)
        assert_eq!(list[0].time_ms, 6000);
        assert_eq!(list[4].index, 1); // oldest (date=1, 5000ms)
        assert_eq!(list[4].time_ms, 5000);
    }
}
