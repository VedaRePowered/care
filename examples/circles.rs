#[care::draw]
fn draw() {
    care::graphics::circle((100, 100), 20);
    care::graphics::rectangle_rounded((250, 300), (100, 50), 0, [0; 4]);
}

care::main!();
