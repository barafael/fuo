use std::{pin::Pin, time::Duration};

use futures::{StreamExt, stream::FuturesUnordered};
use tokio::{sync::mpsc, time::sleep};

enum Effect {
    MakeNew(i32, u64),
    Finish(i32),
}

type Bag = FuturesUnordered<Pin<Box<dyn Future<Output = i32> + Send>>>;

pub async fn event_loop(mut rx: mpsc::Receiver<(i32, u64)>) {
    let mut fuo: Bag = FuturesUnordered::new();
    loop {
        let effect = tokio::select! {
            msg = rx.recv() => {
                let Some((id, seconds)) = msg else {
                    break;
                };
                Effect::MakeNew(id, seconds)
            },
            id = fuo.next(), if !fuo.is_empty() => {
                let Some(id) = id else {
                    panic!("fuo is not empty, but next yielded None");
                };
                Effect::Finish(id)
            }
        };
        match effect {
            Effect::MakeNew(id, seconds) => {
                fuo.push(Box::pin(async move {
                    println!("Received message: {:?}", (id, seconds));
                    sleep(Duration::from_secs(seconds)).await;
                    id
                }));
            }
            Effect::Finish(id) => {
                println!("completed. Id: {id:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{StreamExt, stream::FuturesUnordered};
    use tokio::time::{Instant, Sleep, sleep};
    use tokio_test::assert_elapsed;

    #[tokio::test(start_paused = true)]
    async fn what_does_an_empty_fuo_yield() {
        let start = Instant::now();
        let mut fuo: FuturesUnordered<Sleep> = FuturesUnordered::new();
        assert!(fuo.next().await.is_none());
        fuo.push(sleep(Duration::from_secs(1)));
        assert_eq!(fuo.next().await, Some(()));
        assert!(fuo.next().await.is_none());
        assert!(fuo.next().await.is_none());
        fuo.push(sleep(Duration::from_secs(1)));
        assert_eq!(fuo.next().await, Some(()));
        assert_elapsed!(start, Duration::from_secs(2));
    }

    #[tokio::test]
    #[should_panic = "not yet implemented"]
    async fn async_todo() {
        let fut = async { todo!() };
        tokio::time::sleep(Duration::from_secs(5)).await;
        fut.await;
    }
}
