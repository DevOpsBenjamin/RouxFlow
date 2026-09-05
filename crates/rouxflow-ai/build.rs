use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tables/fb_table.bin");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let table_path = Path::new(&manifest_dir).join("tables/fb_table.bin");

    if !table_path.exists() {
        panic!(
            "\n\n\
            ========================================\n\
            Missing pruning table: tables/fb_table.bin\n\
            Run:  pnpm build:table\n\
            ========================================"
        );
    }
}
