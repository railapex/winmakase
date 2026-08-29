//! Diagnostic: run the supervisor's process discovery against a real image
//! path. `cargo run -p winsome --example procfind -- <full-exe-path>`
fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: procfind <full-exe-path>");
    match winsome::procs::pids_for_image_path(&path) {
        Ok(pids) if pids.is_empty() => println!("no process running from {path}"),
        Ok(pids) => println!("{path}: pids {pids:?}"),
        Err(e) => println!("discovery failed: {e}"),
    }
}
