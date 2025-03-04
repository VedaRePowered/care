use care::math::Vec2;

async fn secondary() {
    println!("hi!");
    for _i in 1..100 {
        care::event::next_frame().await;
    }
    println!("its been 100 frames!");
}

fn main() {
    care::window::open("uwu");
    care::event::main_async_manual(Box::pin(async {
        care::event::spawn(secondary());

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

            care::event::next_frame().await;
        }
    }));
}
