use care::prelude::*;

#[care::state]
struct State {
    texture: Texture,
    pos: Vec2,
}

#[care::init]
fn init() -> State {
    State {
        texture: Texture::new("examples/test.png"),
        pos: Vec2::new(400, 300),
    }
}

#[care::update]
fn update(state: &mut State, delta: f32) {
    if care::keyboard::is_down(Key::Up) {
        state.pos.0.y -= delta*200.0;
    }
    if care::keyboard::is_down(Key::Down) {
        state.pos.0.y += delta*200.0;
    }
    if care::keyboard::is_down(Key::Right) {
        state.pos.0.x += delta*200.0;
    }
    if care::keyboard::is_down(Key::Left) {
        state.pos.0.x -= delta*200.0;
    }
}

#[care::draw]
fn draw(state: &State) {
    care::graphics::texture(&state.texture, state.pos-state.texture.size()/2.0);
}

care::main!();
