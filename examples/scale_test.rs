#[care::draw]
fn draw() {
    care::graphics::set_colour((1, 0, 0, 1));
    care::graphics::rectangle((-8192, -8192), (8192, 8192));
    let size = care::window::window_size();
    care::graphics::set_colour((0.2, 0.2, 0.2, 1));
    care::graphics::rectangle((0, 0), size);
    care::graphics::set_colour((1, 1, 1, 1));
    care::graphics::rectangle_line((0, 0), size, 2);
    care::graphics::line_segment((size.x/2.0, 0), (size.x/2.0, size.y), 2);
    care::graphics::line_segment((0, size.y/2.0), (size.x, size.y/2.0), 2);
    care::graphics::circle(care::mouse::get_position(), 5);
}

care::main!();
