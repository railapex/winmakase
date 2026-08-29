//! Bench: close→relayout latency over GlazeWM IPC.
//!
//! `cargo run -p winmakase --release --example relayout_bench -- <survivor-id> <victim-id>`
//!
//! Closes the victim window and polls until the survivor's rect changes —
//! the moment the WM has re-laid the row out. Poll resolution is one IPC
//! round trip (~2-5ms), fine for the ~100ms effect being measured
//! (fork-ladder A/B: stock 3.10.1 vs KhangHLe's batched repositions).

use std::time::Instant;

use winmakase::config::Config;
use winmakase::glazewm::{Client, Node, Workspace};

fn rect_of(workspaces: &[Workspace], id: &str) -> Option<(i64, i64, i64, i64)> {
    fn find<'a>(children: &'a [Node], id: &str) -> Option<&'a Node> {
        for c in children {
            if c.id == id {
                return Some(c);
            }
            if let Some(f) = find(&c.children, id) {
                return Some(f);
            }
        }
        None
    }
    for ws in workspaces {
        if let Some(n) = find(&ws.children, id) {
            return Some((n.x?, n.y?, n.width?, n.height?));
        }
    }
    None
}

fn main() {
    let mut args = std::env::args().skip(1);
    let survivor = args
        .next()
        .expect("usage: relayout_bench <survivor-id> <victim-id>");
    let victim = args
        .next()
        .expect("usage: relayout_bench <survivor-id> <victim-id>");

    let cfg = Config::default();
    let client = Client::from_config(&cfg).expect("connect to GlazeWM");

    let before =
        rect_of(&client.query_workspaces().unwrap(), &survivor).expect("survivor not found");

    let t0 = Instant::now();
    client
        .command_for(&victim, &["close"])
        .expect("close the victim");

    loop {
        let ws = client.query_workspaces().unwrap();
        if let Some(now) = rect_of(&ws, &survivor) {
            if now != before {
                println!("{}", t0.elapsed().as_millis());
                return;
            }
        } else {
            // Survivor gone too?! Report loudly instead of spinning.
            eprintln!("survivor disappeared");
            std::process::exit(2);
        }
        if t0.elapsed().as_secs() > 10 {
            eprintln!("no relayout within 10s");
            std::process::exit(3);
        }
    }
}
