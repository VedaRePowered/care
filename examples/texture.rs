use care::graphics::Texture;

#[care::state]
static tex: Texture = Texture::new("examples/test.png");

#[care::init]
fn init(_args: Vec<String>) {}

#[care::draw]
fn draw() {
    care::graphics::texture(&tex, (50, 50));
}

care::main!();
