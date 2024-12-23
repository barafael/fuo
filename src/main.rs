use std::io::{BufRead, stdin};

use fuo::event_loop;
use tokio::{sync::mpsc, try_join};

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(100);
    let fuo_processor = tokio::spawn(event_loop(rx));

    let stdin = tokio::task::spawn_blocking(move || {
        let mut lines = stdin().lock().lines();
        loop {
            let Some(Ok(line)) = lines.next() else {
                break;
            };
            let mut items = line.split_whitespace();
            let Some(Ok(id)) = items.next().map(|s| s.parse::<i32>()) else {
                continue;
            };
            let Some(Ok(seconds)) = items.next().map(|s| s.parse::<u64>()) else {
                continue;
            };

            let Ok(()) = tx.try_send((id, seconds)) else {
                break;
            };
        }
    });

    try_join!(fuo_processor, stdin).unwrap();
}
