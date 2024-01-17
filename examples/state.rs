// This example shows off how to add "global" state to the application.
// Due to the way this is implemented, the state variables are only accesable in the init, update and draw functions.

#[care::state]
static time: f32 = 0.0;

#[care::update]
fn update(delta: f32) {
    time += delta;
}

#[care::draw]
fn draw() {
    care::graphics::rectangle((50, 50), (50.0*time.sin(), 50.0*time.cos()));
}

care::main!();
