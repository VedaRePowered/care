#[care::draw]
fn draw() {
    // Random amount, colours, and sizes
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _i in 0..rng.gen_range(5000..10000) {
        care::graphics::set_colour((rng.gen_range(0.0f32..1.0), rng.gen_range(0.0f32..1.0), rng.gen_range(0.0f32..1.0), 1.0));
        care::graphics::rectangle((rng.gen_range(0..750), rng.gen_range(0..550)), (rng.gen_range(10..50), rng.gen_range(10..50)));
    }
}

care::main!();

