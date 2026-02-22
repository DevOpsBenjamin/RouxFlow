use rouxflow_core::stats::{
    compute_ao_n, compute_best_ao_n, compute_tps, compute_session_stats, compute_solve_list,
    trimmed_mean,
};
use rouxflow_core::session::{Solve, SessionType};

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
        integrity: None,
        reaction_ms: None,
        putdown_delay_ms: None,
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
        integrity: None,
        reaction_ms: None,
        putdown_delay_ms: None,
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
