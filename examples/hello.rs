// You might already notice the similarities with love 2d

#[care::draw]
fn draw() {
    care::graphics::text("Hello, World!", (20, 20));
}

// This creates a default main function that calls the draw, update and init functions if present,
// as well as processing events and displaying the framebuffer every frame. You can pass a
// care::Conf, or a function returning care::Conf, to this function to configure care.
care::main!();
