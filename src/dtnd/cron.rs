use core::future::Future;
use std::time::Duration;

use futures_util::StreamExt;
use smol::Timer;

pub async fn spawn_timer<F, Fut>(time_interval: Duration, f: F)
    where
        F: Fn() -> Fut,
        Fut: Future,
{
    let mut task = Timer::interval(time_interval);
    loop {
        task.next().await;
        f().await;
    }
}
