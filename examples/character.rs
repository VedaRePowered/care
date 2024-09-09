use care::prelude::*;

#[care::state]
static texture: Texture = Texture::new("examples/test.png");

#[care::state]
static pos: Vec2 = Vec2::new(400, 300);

#[care::update]
fn update(delta: f32) {
    if care::keyboard::is_down(Key::Up) {
        pos.0.y -= delta * 200.0;
    }
    if care::keyboard::is_down(Key::Down) {
        pos.0.y += delta * 200.0;
    }
    if care::keyboard::is_down(Key::Right) {
        pos.0.x += delta * 200.0;
    }
    if care::keyboard::is_down(Key::Left) {
        pos.0.x -= delta * 200.0;
    }
}

#[care::draw]
fn draw() {
    care::graphics::texture(&texture, pos - texture.size() / 2.0);
}

care::main!();
