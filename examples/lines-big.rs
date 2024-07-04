use care::graphics::{LineJoinStyle, LineEndStyle};

#[care::draw]
fn draw() {
    care::window::set_window_size((1200, 1080));
    care::graphics::set_colour((0.5, 0.5, 0.5, 1));
    care::graphics::text("None", (20, 10));
    care::graphics::text("Merge", (220, 10));
    care::graphics::text("Miter", (420, 10));
    care::graphics::text("Miter (Unlimited)", (620, 10));
    care::graphics::text("Bevel", (820, 10));
    care::graphics::text("Rounded", (1020, 10));
    care::graphics::set_colour((1, 1, 1, 1));
    for (x, line_style) in [(0, LineJoinStyle::None), (200, LineJoinStyle::Merge), (400, LineJoinStyle::Miter), (600, LineJoinStyle::MiterUnlimited), (800, LineJoinStyle::Bevel), (1000, LineJoinStyle::Rounded)] {
        care::graphics::set_line_style(line_style, LineEndStyle::Flat);
        care::graphics::line([(x+50, 100), (x+100, 100), (x+150, 100)], 20);
        care::graphics::line([(x+50, 260), (x+100, 280), (x+150, 260)], 20);
        care::graphics::line([(x+50, 470), (x+100, 470), (x+150, 420)], 20);
        care::graphics::line([(x+50, 660), (x+150, 660), (x+150, 560)], 20);
        care::graphics::line([(x+50, 730), (x+150, 780), (x+50, 830)], 20);
        care::graphics::line([(x+25, 970), (x+125, 955), (x+25, 940)], 20);
    }
    for x in 1..=6 {
        care::graphics::line_segment((0, x*170+15), (1200, x*170+15), 2);
        care::graphics::line_segment((x*200, 0), (x*200, 1080), 2);
    }
}

care::main!();
