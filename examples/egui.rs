use std::sync::Arc;

use parking_lot::Mutex;
use care::prelude::*;

#[care::async_main]
async fn main() {
    let pos = Arc::new(Mutex::new(Vec2::new(100, 100)));
    loop {
        // do the update and draw stuff
        care::graphics::rectangle(*pos.lock(), (100, 100));

        if care::keyboard::is_down('d') {
            pos.lock().x += 1.0;
        }
        if care::keyboard::is_down('a') {
            pos.lock().x -= 1.0;
        }
        if care::keyboard::is_down('s') {
            pos.lock().y += 1.0;
        }
        if care::keyboard::is_down('w') {
            pos.lock().y -= 1.0;
        }

        let pos = pos.clone();
        care::gui::window("Test Gui", move |_ctx, ui| {
            if ui.button("Reset").clicked() {
                *pos.lock() = Vec2::new(100, 100);
                println!("Foo!");
            }
        });

        care::event::next_frame().await;
    }
}

care::main!();
