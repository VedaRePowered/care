use care::prelude::*;

#[care::async_main]
async fn main() {
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

        care::gui::window("Test Gui", |_ctx, ui| {
            if ui.button("Reset").clicked() {
                //pos = Vec2::new(100, 100);
            }
        });

        care::event::next_frame().await;
    }
}

care::main!();
