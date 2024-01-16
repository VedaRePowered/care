use care::prelude::*;

#[care::state]
static texture: Texture = Texture::new("test.png");

#[care::state]
static pos: Vec2 = Vec2::new(400, 300);

#[care::update]
fn update(delta: f32) {
    if care::keyboard::is_down(Key::Up) {
        pos.y += delta*100.0;
    }
    if care::keyboard::is_down(Key::Down) {
        pos.y -= delta*100.0;
    }
    if care::keyboard::is_down(Key::Right) {
        pos.x += delta*100.0;
    }
    if care::keyboard::is_down(Key::Left) {
        pos.x -= delta*100.0;
    }
}

#[care::draw]
fn draw() {
    care::graphics::texture(texture, pos);
}
