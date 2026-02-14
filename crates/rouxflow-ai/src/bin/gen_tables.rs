//! Standalone binary to generate AI pruning tables.
//! Run via: `pnpm build:table` or `cargo run -p rouxflow-ai --bin gen-tables`
//!
//! Writes tables to `crates/rouxflow-ai/tables/` which are then included
//! at compile time via `include_bytes!`.

use rouxflow_ai::pruning::PruningTable;
use std::path::Path;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let tables_dir = Path::new(manifest_dir).join("tables");
    std::fs::create_dir_all(&tables_dir).unwrap();

    let dest = tables_dir.join("fb_table.bin");
    println!("Generating FB pruning table -> {}", dest.display());

    let mut table = PruningTable {
        table: vec![255; PruningTable::FB_SIZE],
        size: PruningTable::FB_SIZE,
    };
    table.generate_fb_table();
    table.save(dest.to_str().unwrap()).unwrap();

    println!("Done! Table written to {}", dest.display());
}
