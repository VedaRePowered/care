use parking_lot::RwLock;

use care::graphics::Texture;

static tex: RwLock<Option<Texture>> = RwLock::new(None);

#[care::init]
fn init(_args: Vec<String>) {
    let texture = Texture::new("examples/test.png");
    *tex.write() = Some(texture);
}

#[care::draw]
fn draw() {
    care::graphics::texture(tex.read().as_ref().unwrap(), (50, 50));
}

care::main!();
