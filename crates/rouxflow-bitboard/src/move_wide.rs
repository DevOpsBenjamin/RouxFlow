use crate::bitcube::BitCube;
use crate::move_indices::WideMove;

impl BitCube {
    /// Applies a wide move (move face + adjacent slice).
    pub fn apply_wide_move(&mut self, m: WideMove) {
        match m {
            WideMove::Uw => self.wide_uw(),
            WideMove::Uwp => self.wide_uw_prime(),
            WideMove::Uw2 => self.wide_uw2(),

            WideMove::Dw => self.wide_dw(),
            WideMove::Dwp => self.wide_dw_prime(),
            WideMove::Dw2 => self.wide_dw2(),

            WideMove::Lw => self.wide_lw(),
            WideMove::Lwp => self.wide_lw_prime(),
            WideMove::Lw2 => self.wide_lw2(),

            WideMove::Rw => self.wide_rw(),
            WideMove::Rwp => self.wide_rw_prime(),
            WideMove::Rw2 => self.wide_rw2(),

            WideMove::Fw => self.wide_fw(),
            WideMove::Fwp => self.wide_fw_prime(),
            WideMove::Fw2 => self.wide_fw2(),

            WideMove::Bw => self.wide_bw(),
            WideMove::Bwp => self.wide_bw_prime(),
            WideMove::Bw2 => self.wide_bw2(),
        }
    }

    // Wide moves are defined as a face rotation plus the corresponding slice rotation.
    // Note: E/M/S Directions follow specific faces:
    // Uw move = U + E' (because E follows D)
    // Dw move = D + E
    // Lw move = L + M
    // Rw move = R + M' (because M follows L)
    // Fw move = F + S
    // Bw move = B + S' (because S follows F)

    pub fn wide_uw(&mut self) {
        self.face_u();
        self.slice_e_prime();
    }
    pub fn wide_dw(&mut self) {
        self.face_d();
        self.slice_e();
    }
    pub fn wide_lw(&mut self) {
        self.face_l();
        self.slice_m();
    }
    pub fn wide_rw(&mut self) {
        self.face_r();
        self.slice_m_prime();
    }
    pub fn wide_fw(&mut self) {
        self.face_f();
        self.slice_s();
    }
    pub fn wide_bw(&mut self) {
        self.face_b();
        self.slice_s_prime();
    }

    pub fn wide_uw_prime(&mut self) {
        self.face_u_prime();
        self.slice_e();
    }
    pub fn wide_dw_prime(&mut self) {
        self.face_d_prime();
        self.slice_e_prime();
    }
    pub fn wide_lw_prime(&mut self) {
        self.face_l_prime();
        self.slice_m_prime();
    }
    pub fn wide_rw_prime(&mut self) {
        self.face_r_prime();
        self.slice_m();
    }
    pub fn wide_fw_prime(&mut self) {
        self.face_f_prime();
        self.slice_s_prime();
    }
    pub fn wide_bw_prime(&mut self) {
        self.face_b_prime();
        self.slice_s();
    }

    pub fn wide_uw2(&mut self) {
        self.wide_uw();
        self.wide_uw();
    }
    pub fn wide_dw2(&mut self) {
        self.wide_dw();
        self.wide_dw();
    }
    pub fn wide_lw2(&mut self) {
        self.wide_lw();
        self.wide_lw();
    }
    pub fn wide_rw2(&mut self) {
        self.wide_rw();
        self.wide_rw();
    }
    pub fn wide_fw2(&mut self) {
        self.wide_fw();
        self.wide_fw();
    }
    pub fn wide_bw2(&mut self) {
        self.wide_bw();
        self.wide_bw();
    }
}
