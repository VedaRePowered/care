use care::prelude::*;

#[care::async_main]
async fn main() {
    let mut pos = Vec2::new(100, 100);
    loop {
        // do the update and draw stuff
        care::graphics::rectangle(pos, (100, 100));

        if care::keyboard::is_down('d') {
            pos.x += 1.0;
        }
        if care::keyboard::is_down('a') {
            pos.x -= 1.0;
        }
        if care::keyboard::is_down('s') {
            pos.y += 1.0;
        }
        if care::keyboard::is_down('w') {
            pos.y -= 1.0;
        }

        care::gui::gui(|ctx| {
            care::gui::Window::new("Test").show(ctx, |ui| {
                if ui.button("Reset").clicked() {
                    pos.x = 100.0;
                    pos.y = 100.0;
                    println!("Foo!");
                }
            });
        });

        care::event::next_frame().await;
    }
}

care::main!();
