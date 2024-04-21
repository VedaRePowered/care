use std::{future::PollFn, task::Poll};

use care::math::Vec2;

fn main() {
    care::window::init();
    care::window::open("uwu");
    care::graphics::init();
    care::event::main_async(Box::pin(async {
        let mut pos = Vec2::new(100, 100);
        loop {
            // do the update and draw stuff
            care::graphics::rectangle(pos, (100, 100));

            if care::keyboard::is_down('d') {
                pos.0.x += 1.0;
            }
            if care::keyboard::is_down('a') {
                pos.0.x -= 1.0;
            }
            if care::keyboard::is_down('s') {
                pos.0.y += 1.0;
            }
            if care::keyboard::is_down('w') {
                pos.0.y -= 1.0;
            }

            care::graphics::present();
            care::keyboard::reset();
            care::mouse::reset();

            let mut ready = false;
            std::future::poll_fn(move |_| {
                if ready {
                    Poll::Ready(())
                } else {
                    ready = true;
                    Poll::Pending
                }
            })
            .await;

            let _ = ::std::thread::sleep(::std::time::Duration::from_millis(1));
        }
    }));
}
