use crate::bitcube::BitCube;

impl BitCube {
    // --- Bitmask constants for phase detection ---
    // L face bottom 2 rows (excluding center 40 for pseudo-block compatibility)
    pub const L_BLOCK: u64 = (1<<39)|(1<<41)|(1<<42)|(1<<43)|(1<<44);
    // R face bottom 2 rows: positions 12-17 (excluding center 13)
    pub const R_BLOCK: u64 = (1<<12)|(1<<14)|(1<<15)|(1<<16)|(1<<17);
    
    // D-layer bar masks (left/right columns)
    pub const D_BAR_L: u64 = (1<<27)|(1<<30)|(1<<33);
    pub const D_BAR_R: u64 = (1<<29)|(1<<32)|(1<<35);
    
    // F bar masks (left/right edge stickers)
    pub const F_BAR_L: u64 = (1<<21)|(1<<24);
    pub const F_BAR_R: u64 = (1<<23)|(1<<26);
    
    // B bar masks (left/right edge stickers)
    pub const B_BAR_L: u64 = (1<<50)|(1<<53);
    pub const B_BAR_R: u64 = (1<<48)|(1<<51);
    
    // U-face corner positions (for CMLL)
    pub const U_CORNERS: u64 = (1<<0)|(1<<2)|(1<<6)|(1<<8);
    
    // Side corner pairs (for CMLL permutation)
    pub const SIDE_PAIRS: [u64; 4] = [
        (1<<18)|(1<<20),  // F top corners
        (1<<9)|(1<<11),   // R top corners
        (1<<36)|(1<<38),  // L top corners
        (1<<45)|(1<<47),  // B top corners
    ];
    
    // Face masks for solved check
    pub const FACE_MASKS: [u64; 6] = [
        0x1FF,        // U: 0-8
        0x1FF << 9,   // R: 9-17
        0x1FF << 18,  // F: 18-26
        0x1FF << 27,  // D: 27-35
        0x1FF << 36,  // L: 36-44
        0x1FF << 45,  // B: 45-53
    ];

    /// Helper: Checks if bitmask has uniform color (all bits in mask belong to same color board)
    pub fn is_uniform(&self, mask: u64) -> bool {
        self.boards.iter().any(|&b| (b & mask) == mask)
    }

    /// Check if a 1x2x3 block exists at the standard Left-Bottom position.
    /// Returns true if L-face stickers, F-bar, B-bar, and D-bar are each uniform in color.
    pub fn is_l_block_formed(&self) -> bool {
        if !self.is_uniform(Self::L_BLOCK) { return false; }
        if !self.is_uniform(Self::F_BAR_L) { return false; }
        if !self.is_uniform(Self::B_BAR_L) { return false; }
        if !self.is_uniform(Self::D_BAR_L) { return false; }
        true
    }

    /// Scans the entire cube for ANY 1x2x3 block (2 corners + 3 edges + 1 center).
    /// Returns true if at least one First Block candidate is found.
    pub fn is_fb_block(&self) -> bool {
        let mut temp = self.clone();
        
        let mut rot_x = |c: &mut BitCube| {
            c.rotate_r(); c.rotate_m_prime(); c.rotate_l_prime();
        };
        
        // Scan ring L-F-R-B (y axis)
        for _ in 0..4 {
            for _ in 0..4 { 
               if temp.is_l_block_formed() { return true; }
               rot_x(&mut temp);
            }
            temp.rotate_y();
        }

        // Now bring U to L (z' = F' S' B)
        temp = self.clone();
        temp.rotate_f_prime(); temp.rotate_s_prime(); temp.rotate_b();
        for _ in 0..4 {
           if temp.is_l_block_formed() { return true; }
           rot_x(&mut temp);
        }

        // Now bring D to L (z = F S B')
        temp = self.clone();
        temp.rotate_f(); temp.rotate_s(); temp.rotate_b_prime();
        for _ in 0..4 {
           if temp.is_l_block_formed() { return true; }
           rot_x(&mut temp);
        }

        false
    }

    pub fn is_fb_solved(&self) -> bool {
        self.is_fb_solved_ext(5, 1, 2, 3)
    }

    pub fn is_fb_solved_ext(&self, l: usize, d: usize, f: usize, b: usize) -> bool {
        self.boards[l] & (Self::L_BLOCK | (1<<40)) == (Self::L_BLOCK | (1<<40)) &&
        self.boards[d] & Self::D_BAR_L == Self::D_BAR_L &&
        self.boards[f] & Self::F_BAR_L == Self::F_BAR_L &&
        self.boards[b] & Self::B_BAR_L == Self::B_BAR_L
    }

    pub fn is_sb_solved(&self) -> bool {
        self.is_sb_solved_ext(4, 1, 2, 3)
    }

    pub fn is_sb_solved_ext(&self, r: usize, d: usize, f: usize, b: usize) -> bool {
        self.boards[r] & (Self::R_BLOCK | (1<<13)) == (Self::R_BLOCK | (1<<13)) &&
        self.boards[d] & Self::D_BAR_R == Self::D_BAR_R &&
        self.boards[f] & Self::F_BAR_R == Self::F_BAR_R &&
        self.boards[b] & Self::B_BAR_R == Self::B_BAR_R
    }

    pub fn is_cmll_solved(&self) -> bool {
        if !self.boards.iter().any(|&b| b & Self::U_CORNERS == Self::U_CORNERS) { return false; }
        Self::SIDE_PAIRS.iter().all(|&mask| self.boards.iter().any(|&b| b & mask == mask))
    }

    pub fn is_ul_ur_placed(&self) -> bool {
        let u_corner = self.get_color_at(0); 
        let l_color = self.get_color_at(40);
        let r_color = self.get_color_at(13);
        self.get_color_at(3) == u_corner && self.get_color_at(37) == l_color &&
        self.get_color_at(5) == u_corner && self.get_color_at(10) == r_color
    }

    pub fn is_l4e_solved(&self) -> bool {
        let u_c = self.get_color_at(4);
        let f_c = self.get_color_at(22);
        let d_c = self.get_color_at(31);
        let b_c = self.get_color_at(49);
        self.get_color_at(7) == u_c && self.get_color_at(19) == f_c &&  
        self.get_color_at(1) == u_c && self.get_color_at(46) == b_c &&  
        self.get_color_at(28) == d_c && self.get_color_at(25) == f_c && 
        self.get_color_at(34) == d_c && self.get_color_at(52) == b_c    
    }

    pub fn count_bad_edges(&self) -> usize {
        let u_ref = self.get_color_at(4);
        let f_ref = self.get_color_at(22);
        let edges = [(7, 19), (1, 46), (31, 25), (34, 52), (3, 37), (5, 10)];
        let mut bad = 0;
        for (st1, st2) in edges {
            let (c1, c2) = (self.get_color_at(st1), self.get_color_at(st2));
            if Self::is_bad_edge(c1, c2, u_ref, f_ref) { bad += 1; }
        }
        bad
    }

    pub fn is_solved(&self) -> bool {
        Self::FACE_MASKS.iter().all(|&mask| self.boards.iter().any(|&b| b & mask == mask))
    }

    fn is_bad_edge(top_front: usize, side: usize, u_ref: usize, f_ref: usize) -> bool {
        let u_opp = u_ref ^ 1; 
        let f_opp = f_ref ^ 1;
        if side == u_ref || side == u_opp { return true; }
        if (side == f_ref || side == f_opp) && (top_front == u_ref || top_front == u_opp) {
            return false;
        }
        false
    }
}
