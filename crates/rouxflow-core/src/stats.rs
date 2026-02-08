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
}

#[derive(Serialize)]
pub struct SolveListEntry {
    pub id: String,
    pub index: usize,
    pub time_ms: u32,
    pub turns: usize,
    pub tps: f64,
    pub is_best: bool,
}

/// Trimmed mean of `n` values: remove best and worst, average the rest.
/// Returns None if fewer than 3 values (need at least 3 to trim).
pub fn compute_ao_n(times: &[u32], n: usize) -> Option<u32> {
    if times.len() < n || n < 3 {
        return None;
    }
    let window = &times[times.len() - n..];
    trimmed_mean(window)
}

/// Slide a window of size `n` across all times, return the lowest trimmed mean.
pub fn compute_best_ao_n(times: &[u32], n: usize) -> Option<u32> {
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

fn trimmed_mean(window: &[u32]) -> Option<u32> {
    if window.len() < 3 {
        return None;
    }
    let mut sorted: Vec<u32> = window.to_vec();
    sorted.sort_unstable();
    // Remove best (first) and worst (last), average the rest
    let trimmed = &sorted[1..sorted.len() - 1];
    let sum: u64 = trimmed.iter().map(|&t| t as u64).sum();
    Some((sum / trimmed.len() as u64) as u32)
}

pub fn compute_tps(solve: &Solve) -> f64 {
    if solve.time == 0 {
        return 0.0;
    }
    solve.moves.len() as f64 / (solve.time as f64 / 1000.0)
}

pub fn compute_session_stats(solves: &[Solve], session_type: SessionType) -> SessionStats {
    // Sort by date to ensure chronological order (current Ao5 = last 5 chronologically)
    let mut valid: Vec<&Solve> = solves.iter().filter(|s| s.is_valid).collect();
    valid.sort_by_key(|s| s.date);
    let times: Vec<u32> = valid.iter().map(|s| s.time).collect();

    let best_ms = times.iter().copied().min();
    let worst_ms = times.iter().copied().max();
    let average_ms = if times.is_empty() {
        None
    } else {
        let sum: u64 = times.iter().map(|&t| t as u64).sum();
        Some((sum / times.len() as u64) as u32)
    };

    let current_ao5_ms = compute_ao_n(&times, 5);
    let current_ao12_ms = compute_ao_n(&times, 12);
    let best_ao5_ms = compute_best_ao_n(&times, 5);
    let best_ao12_ms = compute_best_ao_n(&times, 12);

    let mean_tps = if valid.is_empty() {
        None
    } else {
        let tps_sum: f64 = valid.iter().map(|s| compute_tps(s)).sum();
        Some(tps_sum / valid.len() as f64)
    };

    let session_type_str = match session_type {
        SessionType::Free => "Free".to_string(),
        SessionType::WCA => "WCA".to_string(),
    };

    SessionStats {
        solve_count: valid.len(),
        best_ms,
        worst_ms,
        average_ms,
        current_ao5_ms,
        current_ao12_ms,
        best_ao5_ms,
        best_ao12_ms,
        mean_tps,
        session_type: session_type_str,
    }
}

pub fn compute_solve_list(solves: &[Solve]) -> Vec<SolveListEntry> {
    // Sort by date to ensure chronological order
    let mut sorted: Vec<&Solve> = solves.iter().filter(|s| s.is_valid).collect();
    sorted.sort_by_key(|s| s.date);

    let best_time = sorted.iter().map(|s| s.time).min();

    let mut entries: Vec<SolveListEntry> = sorted
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let tps = compute_tps(s);
            SolveListEntry {
                id: s.id.clone(),
                index: i + 1,
                time_ms: s.time,
                turns: s.moves.len(),
                tps,
                is_best: best_time == Some(s.time),
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
        }
    }

    fn make_solve(id: &str, time: u32, moves: usize) -> Solve {
        make_solve_at(id, time, moves, 0)
    }

    #[test]
    fn test_trimmed_mean() {
        // [1000, 2000, 3000, 4000, 5000] → trim best(1000) and worst(5000) → avg(2000,3000,4000) = 3000
        assert_eq!(trimmed_mean(&[1000, 2000, 3000, 4000, 5000]), Some(3000));
    }

    #[test]
    fn test_compute_ao_n() {
        let times = vec![5000, 4000, 3000, 2000, 1000];
        // Last 5: [5000,4000,3000,2000,1000] → trim 1000,5000 → avg(4000,3000,2000) = 3000
        assert_eq!(compute_ao_n(&times, 5), Some(3000));
    }

    #[test]
    fn test_compute_ao_n_insufficient() {
        let times = vec![1000, 2000];
        assert_eq!(compute_ao_n(&times, 5), None);
    }

    #[test]
    fn test_compute_best_ao_n() {
        // 6 solves: [5000, 4000, 3000, 2000, 1000, 1500]
        // Window 0..5: [5000,4000,3000,2000,1000] → trim → avg(4000,3000,2000)=3000
        // Window 1..6: [4000,3000,2000,1000,1500] → trim → avg(3000,2000,1500)=2166
        let times = vec![5000, 4000, 3000, 2000, 1000, 1500];
        assert_eq!(compute_best_ao_n(&times, 5), Some(2166));
    }

    #[test]
    fn test_compute_tps() {
        let solve = make_solve("1", 10000, 50); // 10s, 50 moves = 5 TPS
        let tps = compute_tps(&solve);
        assert!((tps - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_session_stats_empty() {
        let stats = compute_session_stats(&[], SessionType::Free);
        assert_eq!(stats.solve_count, 0);
        assert_eq!(stats.best_ms, None);
        assert_eq!(stats.current_ao5_ms, None);
    }

    #[test]
    fn test_session_stats_with_solves() {
        let solves: Vec<Solve> = (1..=5).map(|i| make_solve(&i.to_string(), i * 1000, 20)).collect();
        let stats = compute_session_stats(&solves, SessionType::Free);
        assert_eq!(stats.solve_count, 5);
        assert_eq!(stats.best_ms, Some(1000));
        assert_eq!(stats.worst_ms, Some(5000));
        assert_eq!(stats.average_ms, Some(3000));
        assert_eq!(stats.current_ao5_ms, Some(3000)); // trim best+worst → avg(2000,3000,4000)
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

        let stats = compute_session_stats(&solves, SessionType::Free);

        // Current Ao5 = last 5 = [7500, 8300, 9400, 7600] wait that's 4...
        // Last 5 chronologically: [9100, 7500, 8300, 9400, 7600]
        // sorted: [7500, 7600, 8300, 9100, 9400] → trim → avg(7600, 8300, 9100) = 8333
        assert_eq!(stats.current_ao5_ms, Some(8333));

        // Best Ao5: slide all 8 windows, best is window [7200, 8800, 9100, 7500, 8300]
        // sorted: [7200, 7500, 8300, 8800, 9100] → trim → avg(7500, 8300, 8800) = 8200
        assert_eq!(stats.best_ao5_ms, Some(8200));

        // They MUST differ
        assert_ne!(stats.current_ao5_ms, stats.best_ao5_ms);
    }

    #[test]
    fn test_stats_sort_by_date_not_insertion_order() {
        // Simulate IndexedDB returning solves in random (non-chronological) order
        // Chronological order by date: fast, medium, slow, medium, fast
        // But insertion order is scrambled
        let solves = vec![
            make_solve_at("c", 15000, 20, 3), // date=3, slow
            make_solve_at("a", 5000, 20, 1),  // date=1, fast
            make_solve_at("e", 6000, 20, 5),  // date=5, fast
            make_solve_at("b", 10000, 20, 2), // date=2, medium
            make_solve_at("d", 11000, 20, 4), // date=4, medium
        ];

        let stats = compute_session_stats(&solves, SessionType::Free);
        // After sorting by date: [5000, 10000, 15000, 11000, 6000]
        // Ao5 (all 5 sorted by date): trim 5000,15000 → avg(10000,11000,6000) = 9000
        assert_eq!(stats.current_ao5_ms, Some(9000));

        // Verify solve list is also in date order
        let list = compute_solve_list(&solves);
        assert_eq!(list[0].index, 5); // newest first (date=5, 6000ms)
        assert_eq!(list[0].time_ms, 6000);
        assert_eq!(list[4].index, 1); // oldest (date=1, 5000ms)
        assert_eq!(list[4].time_ms, 5000);
    }
}
