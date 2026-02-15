#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quat_conjugate() {
        let q = [0.1, 0.2, 0.3, 0.9];
        let c = quat_conjugate(&q);
        assert_eq!(c, [-0.1, -0.2, -0.3, 0.9]);
    }

    #[test]
    fn test_quat_multiply_identity() {
        let id = [0.0, 0.0, 0.0, 1.0];
        let q = [0.1, 0.2, 0.3, 0.9];
        let result = quat_multiply(&id, &q);
        for i in 0..4 {
            assert!((result[i] - q[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_relative_quaternion_identity() {
        let home = quat_normalize(&[0.01, -0.03, -0.02, 0.99]);
        let rel = relative_quaternion(&home, &home);
        // Should be approximately identity
        assert!(
            (rel[3] - 1.0).abs() < 0.01,
            "w should be ~1.0, got {}",
            rel[3]
        );
        assert!(rel[0].abs() < 0.01);
        assert!(rel[1].abs() < 0.01);
        assert!(rel[2].abs() < 0.01);
    }

    #[test]
    fn test_estimate_orientation_identity() {
        let id = [0.0, 0.0, 0.0, 1.0];
        let (top, front) = estimate_orientation(&id);
        assert_eq!(top, Color::White);
        assert_eq!(front, Color::Green);
    }

    #[test]
    fn test_remap_identity() {
        // W/G = home -> all moves pass through unchanged
        assert_eq!(remap_move("U", Color::White, Color::Green), "U");
        assert_eq!(remap_move("U'", Color::White, Color::Green), "U'");
        assert_eq!(remap_move("R", Color::White, Color::Green), "R");
        assert_eq!(remap_move("F2", Color::White, Color::Green), "F2");
    }

    #[test]
    fn test_find_gyro_before() {
        let samples = vec![
            GyroSample {
                t: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            GyroSample {
                t: 2.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            GyroSample {
                t: 3.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
        ];
        assert_eq!(find_gyro_before(&samples, 2.5).unwrap().t, 2.0);
        assert_eq!(find_gyro_before(&samples, 2.0).unwrap().t, 2.0);
        assert!(find_gyro_before(&samples, 0.5).is_none());
    }

    #[test]
    fn test_find_gyro_after() {
        let samples = vec![
            GyroSample {
                t: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            GyroSample {
                t: 2.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            GyroSample {
                t: 3.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
        ];
        assert_eq!(find_gyro_after(&samples, 1.5).unwrap().t, 2.0);
        assert_eq!(find_gyro_after(&samples, 2.0).unwrap().t, 2.0);
        assert!(find_gyro_after(&samples, 3.5).is_none());
    }

    #[test]
    fn test_orientation_label() {
        assert_eq!(orientation_label(Color::White, Color::Green), "W/G");
        assert_eq!(orientation_label(Color::Yellow, Color::Red), "Y/R");
    }

    #[test]
    fn test_parse_notation() {
        assert_eq!(parse_notation("U"), ("U", ""));
        assert_eq!(parse_notation("U'"), ("U", "'"));
        assert_eq!(parse_notation("R2"), ("R", "2"));
        assert_eq!(parse_notation("F"), ("F", ""));
    }

    #[test]
    fn test_compute_home_basic() {
        // All samples near identity -> home should be near identity
        let samples: Vec<GyroSample> = (0..20)
            .map(|i| GyroSample {
                t: i as f64,
                x: 0.01,
                y: 0.02,
                z: -0.01,
                w: 0.999,
            })
            .collect();
        let home = compute_home(&samples);
        assert!(home[3] > 0.99, "w should be near 1.0, got {}", home[3]);
    }

    #[test]
    fn test_remap_y_rotation() {
        // After y rotation (CW from top): White top, Red front (W/R)
        // Body +Z (F) now points toward Red (+X in home) -> body F = home R
        // Body +X (R) now points toward Blue (-Z in home) -> body R = home B
        // Direction is always preserved (no flip).
        let remapped = remap_move("F", Color::White, Color::Red);
        assert_eq!(remapped, "R", "Body F in W/R should map to home R");

        let remapped = remap_move("R", Color::White, Color::Red);
        assert_eq!(remapped, "B", "Body R in W/R should map to home B");
    }

    #[test]
    fn test_remap_x2_rotation() {
        // After x2: Yellow top, Blue front (Y/B)
        // Body U -> home D, Body F -> home B
        let remapped = remap_move("U", Color::Yellow, Color::Blue);
        assert_eq!(remapped, "D", "Body U in Y/B should map to home D");

        let remapped = remap_move("F", Color::Yellow, Color::Blue);
        assert_eq!(remapped, "B", "Body F in Y/B should map to home B");

        // R stays R (x2 doesn't change left/right axis)
        let remapped = remap_move("R", Color::Yellow, Color::Blue);
        assert_eq!(remapped, "R", "Body R in Y/B should map to home R");
    }

    #[test]
    fn test_remap_z_prime_rotation() {
        // After z' (CCW from front): Red top, Green front (R/G)
        // Body U -> home R, Body D -> home L
        let remapped = remap_move("U", Color::Red, Color::Green);
        assert_eq!(remapped, "R", "Body U in R/G should map to home R");

        let remapped = remap_move("D", Color::Red, Color::Green);
        assert_eq!(remapped, "L", "Body D in R/G should map to home L");

        // Front stays front
        let remapped = remap_move("F", Color::Red, Color::Green);
        assert_eq!(remapped, "F", "Body F in R/G should stay home F");
    }

    #[test]
    fn test_slice_name_body_frame() {
        // BLE directions are negated to get core motion.
        // BLE L(+1) R(-1) = core M' (follows R, opposite of L)
        assert_eq!(slice_name("L", 1, "R", -1), "M'");
        // BLE R(+1) L(-1) = core M (follows L)
        assert_eq!(slice_name("R", 1, "L", -1), "M");
        // BLE F(+1) B(-1) = core S' (opposite of F)
        assert_eq!(slice_name("F", 1, "B", -1), "S'");
        // BLE F(-1) B(+1) = core S (follows F)
        assert_eq!(slice_name("F", -1, "B", 1), "S");
        assert_eq!(slice_name("B", 1, "F", -1), "S");
        // BLE D(+1) U(-1) = core E' (opposite of D)
        assert_eq!(slice_name("D", 1, "U", -1), "E'");
        // BLE U(+1) D(-1) = core E (follows D)
        assert_eq!(slice_name("U", 1, "D", -1), "E");
    }

    #[test]
    fn test_remap_slice_yr() {
        // In Y/R: BLE F'(-1)+B(+1) = core S in body frame → home M'
        // (body S follows body F = home R → slice follows R = home M')
        let home_slice = remap_slice("F'", "B", Color::Yellow, Color::Red);
        assert_eq!(
            home_slice, "M'",
            "Body core-S in Y/R should remap to home M'"
        );

        // In Y/R: BLE B(+1)+F'(-1) same pair, still = home M'
        let home_slice = remap_slice("B", "F'", Color::Yellow, Color::Red);
        assert_eq!(home_slice, "M'", "Body B+F' in Y/R should remap to home M'");
    }

    #[test]
    fn test_remap_slice_identity() {
        // In W/G (home): BLE L(+1)+R'(-1) = core M' stays M'
        let home_slice = remap_slice("L", "R'", Color::White, Color::Green);
        assert_eq!(home_slice, "M'");
    }

    #[test]
    fn test_is_slice_pair_detection() {
        use rouxflow_core::move_interpreter::MoveKind;
        use rouxflow_core::telemetry::RawMove;

        let m1 = RawMove {
            n: "F'".to_string(),
            t: 86.101,
            k: MoveKind::Face,
        };
        let m2 = RawMove {
            n: "B".to_string(),
            t: 86.101,
            k: MoveKind::Face,
        };
        assert!(is_slice_pair(&m1, &m2));

        // Same direction = not a slice
        let m3 = RawMove {
            n: "F".to_string(),
            t: 86.101,
            k: MoveKind::Face,
        };
        let m4 = RawMove {
            n: "B".to_string(),
            t: 86.101,
            k: MoveKind::Face,
        };
        assert!(!is_slice_pair(&m3, &m4));

        // 2ms jitter = still a slice
        let m5 = RawMove {
            n: "F'".to_string(),
            t: 86.101,
            k: MoveKind::Face,
        };
        let m6 = RawMove {
            n: "B".to_string(),
            t: 86.103,
            k: MoveKind::Face,
        };
        assert!(is_slice_pair(&m5, &m6));

        // Different timestamps (>5ms) = not a slice
        let m7 = RawMove {
            n: "F'".to_string(),
            t: 86.101,
            k: MoveKind::Face,
        };
        let m8 = RawMove {
            n: "B".to_string(),
            t: 86.500,
            k: MoveKind::Face,
        };
        assert!(!is_slice_pair(&m7, &m8));
    }
}
