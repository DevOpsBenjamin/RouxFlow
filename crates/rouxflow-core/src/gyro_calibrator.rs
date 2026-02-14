use crate::cube::Quaternion;

/// Accumulates gyro quaternion samples during scrambling and computes
/// a "home" orientation. The conjugate of this home quaternion is used
/// as a renderer offset so the 3D cube shows the expected green-front view.
pub struct GyroCalibrator {
    sum: [f64; 4],          // (x, y, z, w) running sum
    first: Option<[f32; 4]>, // first sample for sign-flip detection
    count: u32,
    active: bool,
    home: Option<Quaternion>,
}

impl GyroCalibrator {
    pub fn new() -> Self {
        Self {
            sum: [0.0; 4],
            first: None,
            count: 0,
            active: false,
            home: None,
        }
    }

    /// Begin accumulating samples. Clears previous data.
    pub fn start(&mut self) {
        self.sum = [0.0; 4];
        self.first = None;
        self.count = 0;
        self.active = true;
        self.home = None;
    }

    /// Feed a quaternion sample. Only accumulates while active.
    /// Handles sign-flip: if dot(sample, first) < 0, negates sample before accumulating.
    pub fn feed(&mut self, q: &Quaternion) {
        if !self.active {
            return;
        }

        let (mut x, mut y, mut z, mut w) = (q.x, q.y, q.z, q.w);

        match self.first {
            None => {
                self.first = Some([x, y, z, w]);
            }
            Some(f) => {
                let dot = x * f[0] + y * f[1] + z * f[2] + w * f[3];
                if dot < 0.0 {
                    x = -x;
                    y = -y;
                    z = -z;
                    w = -w;
                }
            }
        }

        self.sum[0] += x as f64;
        self.sum[1] += y as f64;
        self.sum[2] += z as f64;
        self.sum[3] += w as f64;
        self.count += 1;
    }

    /// Finalize calibration. Requires >= 10 samples for robustness.
    /// Returns the computed home quaternion, or None if not enough samples.
    pub fn finalize(&mut self) -> Option<Quaternion> {
        self.active = false;

        if self.count < 10 {
            return None;
        }

        let n = self.count as f64;
        let (ax, ay, az, aw) = (
            self.sum[0] / n,
            self.sum[1] / n,
            self.sum[2] / n,
            self.sum[3] / n,
        );

        // Normalize
        let len = (ax * ax + ay * ay + az * az + aw * aw).sqrt();
        if len < 1e-10 {
            return None;
        }

        let q = Quaternion {
            x: (ax / len) as f32,
            y: (ay / len) as f32,
            z: (az / len) as f32,
            w: (aw / len) as f32,
        };

        self.home = Some(q);
        Some(q)
    }

    /// Get the computed home orientation (available after finalize).
    pub fn home(&self) -> Option<&Quaternion> {
        self.home.as_ref()
    }

    /// Compute the renderer offset: conjugate(home) as (x, y, z, w).
    /// This makes the home orientation render as identity (green-front view).
    pub fn compute_render_offset(&self) -> Option<(f32, f32, f32, f32)> {
        self.home.map(|h| (-h.x, -h.y, -h.z, h.w))
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for GyroCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion { x, y, z, w }
    }

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn test_basic_calibration() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        // Feed 20 identical quaternions
        let sample = q(0.0, 0.0, 0.3827, 0.9239); // ~45° around z
        for _ in 0..20 {
            cal.feed(&sample);
        }

        let home = cal.finalize().unwrap();
        assert!(approx_eq(home.x, sample.x));
        assert!(approx_eq(home.y, sample.y));
        assert!(approx_eq(home.z, sample.z));
        assert!(approx_eq(home.w, sample.w));
    }

    #[test]
    fn test_sign_flip_handled() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.0, 0.0, 0.3827, 0.9239);
        let negated = q(-sample.x, -sample.y, -sample.z, -sample.w);

        // Feed half q, half -q
        for _ in 0..10 {
            cal.feed(&sample);
        }
        for _ in 0..10 {
            cal.feed(&negated);
        }

        let home = cal.finalize().unwrap();
        // Should still be ~sample (sign-flip corrects the negated ones)
        assert!(approx_eq(home.x, sample.x));
        assert!(approx_eq(home.y, sample.y));
        assert!(approx_eq(home.z, sample.z));
        assert!(approx_eq(home.w, sample.w));
    }

    #[test]
    fn test_min_samples() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..9 {
            cal.feed(&sample);
        }

        assert!(cal.finalize().is_none());
    }

    #[test]
    fn test_not_active_ignores_feed() {
        let mut cal = GyroCalibrator::new();
        // Don't call start()
        let sample = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..20 {
            cal.feed(&sample);
        }

        assert_eq!(cal.count, 0);
        assert!(cal.finalize().is_none());
    }

    #[test]
    fn test_render_offset_is_conjugate() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.1, 0.2, 0.3, 0.9274); // roughly normalized
        for _ in 0..20 {
            cal.feed(&sample);
        }

        cal.finalize().unwrap();
        let (ox, oy, oz, ow) = cal.compute_render_offset().unwrap();
        let home = cal.home().unwrap();

        // offset should be conjugate: (-x, -y, -z, w)
        assert!(approx_eq(ox, -home.x));
        assert!(approx_eq(oy, -home.y));
        assert!(approx_eq(oz, -home.z));
        assert!(approx_eq(ow, home.w));

        // offset * home should be ~identity
        let rw = ow * home.w - ox * home.x - oy * home.y - oz * home.z;
        let rx = ow * home.x + ox * home.w + oy * home.z - oz * home.y;
        let ry = ow * home.y - ox * home.z + oy * home.w + oz * home.x;
        let rz = ow * home.z + ox * home.y - oy * home.x + oz * home.w;

        assert!(approx_eq(rx, 0.0));
        assert!(approx_eq(ry, 0.0));
        assert!(approx_eq(rz, 0.0));
        assert!(approx_eq(rw, 1.0));
    }

    #[test]
    fn test_start_clears_previous() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.5, 0.5, 0.5, 0.5);
        for _ in 0..20 {
            cal.feed(&sample);
        }

        // Start again — previous data should be cleared
        cal.start();
        assert_eq!(cal.count, 0);
        assert!(cal.home.is_none());

        // Not enough samples after restart
        for _ in 0..5 {
            cal.feed(&sample);
        }
        assert!(cal.finalize().is_none());
    }
}
