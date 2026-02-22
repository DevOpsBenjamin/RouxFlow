use crate::cube::Quaternion;
use crate::gyro_snap::AbsoluteStateTracker;

pub struct GyroCalibrator {
    samples: Vec<[f32; 4]>,
    first: Option<[f32; 4]>,
    active: bool,
    home: Option<Quaternion>,
}

impl GyroCalibrator {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            first: None,
            active: false,
            home: None,
        }
    }

    pub fn start(&mut self) {
        self.samples.clear();
        self.first = None;
        self.active = true;
        self.home = None;
    }

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

        self.samples.push([x, y, z, w]);
    }

    pub fn finalize(&mut self) -> Option<Quaternion> {
        self.active = false;

        if self.samples.len() < 10 {
            return None;
        }

        let initial_avg = Self::average_quaternions(&self.samples)?;

        let mut distances: Vec<(usize, f32)> = self
            .samples
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let dot = (s[0] * initial_avg[0]
                    + s[1] * initial_avg[1]
                    + s[2] * initial_avg[2]
                    + s[3] * initial_avg[3])
                    .abs();
                let angle = dot.min(1.0).acos() * 2.0;
                (i, angle)
            })
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let keep_count = (distances.len() as f32 * 0.9) as usize;

        let filtered: Vec<[f32; 4]> = distances[..keep_count]
            .iter()
            .map(|(i, _)| self.samples[*i])
            .collect();

        if filtered.len() < 10 {
            return None;
        }

        let avg = Self::average_quaternions(&filtered)?;

        let home_q = Quaternion {
            x: avg[0],
            y: avg[1],
            z: avg[2],
            w: avg[3],
        };

        self.home = Some(home_q);
        Some(home_q)
    }

    fn average_quaternions(samples: &[[f32; 4]]) -> Option<[f32; 4]> {
        let n = samples.len() as f64;
        if n < 1.0 {
            return None;
        }

        let mut sum = [0.0f64; 4];
        for s in samples {
            sum[0] += s[0] as f64;
            sum[1] += s[1] as f64;
            sum[2] += s[2] as f64;
            sum[3] += s[3] as f64;
        }

        let avg = [sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n];
        let len = (avg[0] * avg[0] + avg[1] * avg[1] + avg[2] * avg[2] + avg[3] * avg[3]).sqrt();
        if len < 1e-10 {
            return None;
        }

        Some([
            (avg[0] / len) as f32,
            (avg[1] / len) as f32,
            (avg[2] / len) as f32,
            (avg[3] / len) as f32,
        ])
    }

    pub fn home(&self) -> Option<&Quaternion> {
        self.home.as_ref()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn compute_render_offset(&self) -> Option<(f32, f32, f32, f32)> {
        self.home.map(|h| (-h.x, -h.y, -h.z, h.w))
    }

    // ====== STUBS TO PREVENT WASM/COMPILER BREAKING ======
    // (Until we completely purge them from other files too)

    pub fn remap_notation(&self, notation: &str) -> String {
        notation.to_string()
    }

    pub fn has_pending_zone_rotation(&self) -> bool {
        false
    }

    pub fn compensate_slice(&mut self, _notation: &str) {
        // Nothing anymore!
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

impl Default for GyroCalibrator {
    fn default() -> Self {
        Self::new()
    }
}
