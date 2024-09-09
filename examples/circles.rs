#[care::draw]
fn draw() {
    care::graphics::circle((100, 100), 30);
    care::graphics::ellipse((400, 100), 20, (2, 0));
    let s2 = (2.0f32).sqrt();
    care::graphics::ellipse((300.0 + s2 * 50.0, 100.0 + s2 * 50.0), 21, (s2, s2));
    care::graphics::ellipse((300, 200), 20, (0, 2));
    care::graphics::rectangle_rounded((250, 300), (100, 50), 0, [0.5; 4]);
}

care::main!();
