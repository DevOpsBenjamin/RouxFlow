use rouxflow_ai::{BitCube, ToFacelet};
use rouxflow_core::cube::facelet::FaceletCube;

#[test]
fn test_50_random_scrambles() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let move_pool = [
        "U", "U'", "U2", "D", "D'", "D2", "L", "L'", "L2", 
        "R", "R'", "R2", "F", "F'", "F2", "B", "B'", "B2",
        "M", "M'", "M2", "S", "S'", "S2", "E", "E'", "E2",
        "r", "r'", "r2", "l", "l'", "l2", "x", "y", "z"
    ];

    println!("\n[TEST] Starting 50 random complex scrambles with full state logging...");
    
    for i in 1..=50 {
        let mut legacy_cube = FaceletCube::new();
        let mut bit_cube = BitCube::new_solved();
        let mut scramble = Vec::new();
        
        let len = rng.gen_range(10..20); // Slightly shorter for readable logs
        for _ in 0..len {
            let m = move_pool[rng.gen_range(0..move_pool.len())];
            scramble.push(m);
            legacy_cube.apply_move(m);
            bit_cube.apply_move(m);
        }

        let bit_as_facelet = bit_cube.to_facelet();
        
        println!("\n--- [SCRAMBLE {:02}/50] ---", i);
        println!("Sequence: {}", scramble.join(" "));
        
        if legacy_cube.facelets != bit_as_facelet.facelets {
            println!("[ERROR] Mismatch detected!");
            debug_mismatch(&legacy_cube, &bit_as_facelet);
            panic!("Randomized test failed at scramble {}", i);
        }
        
        println!("Result: VERIFIED MATCH");
        bit_as_facelet.dump_debug();
    }
    
    println!("\n[SUCCESS] 50/50 random scrambles verified perfectly!");
}

fn debug_mismatch(legacy: &FaceletCube, bit: &FaceletCube) {
    println!("Mismatch analysis (showing first 10 errors):");
    let mut errors = 0;
    for i in 0..54 {
        if legacy.facelets[i] != bit.facelets[i] {
            errors += 1;
            if errors <= 10 {
                let face_names = ["U", "R", "F", "D", "L", "B"];
                let face = i / 9;
                let pos = i % 9;
                println!("    Index {} (Face {}, Pos {}): Legacy={:?}, Bit={:?}", 
                    i, face_names[face], pos, legacy.facelets[i], bit.facelets[i]);
            }
        }
    }
    println!("Total mismatched stickers: {}", errors);
    legacy.dump_debug();
    println!("--- BITBOARD AS FACELET ---");
    bit.dump_debug();
}
