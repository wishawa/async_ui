// use std::pin::Pin;
// use std::task::Context;
// use std::{task::Poll, time::Duration};

// use smol::Timer;
// use std::future::Future;

// struct PendOnce {
//     pended: bool,
// }
// impl PendOnce {
//     fn new() -> Self {
//         Self { pended: false }
//     }
// }

// impl Future for PendOnce {
//     type Output = ();

//     fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
//         if self.pended {
//             Poll::Ready(())
//         } else {
//             self.pended = true;
//             Poll::Pending
//         }
//     }
// }
// async fn test() {
//     loop {
//         println!("polled!");
//         PendOnce::new().await;
//         // yield_now().await;
//     }
// }
// fn main() {
//     smol::block_on(async {
//         let task = smol::spawn(test());
//         println!("spawned");
//         // PendOnce::new().await;
//         Timer::after(Duration::from_secs(2)).await;
//         println!("dropping");
//         drop(task);
//         println!("dropped");
//     });
// }

fn main() {}
