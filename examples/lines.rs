use care::graphics::{LineJoinStyle, LineEndStyle};

#[care::draw]
fn draw() {
    care::graphics::set_colour((0.5, 0.5, 0.5, 1));
    care::graphics::text("None", (10, 5));
    care::graphics::text("Merge", (110, 5));
    care::graphics::text("Miter", (210, 5));
    care::graphics::text("Miter (Unlim)", (310, 5));
    care::graphics::text("Bevel", (410, 5));
    care::graphics::text("Rounded", (510, 5));
    care::graphics::set_colour((1, 1, 1, 1));
    for (x, line_style) in [(0, LineJoinStyle::None), (100, LineJoinStyle::Merge), (200, LineJoinStyle::Miter), (300, LineJoinStyle::MiterUnlimited), (400, LineJoinStyle::Bevel), (500, LineJoinStyle::Rounded)] {
        care::graphics::set_line_style(line_style, LineEndStyle::Flat);
        care::graphics::line([(x+25, 50), (x+50, 50), (x+75, 50)], 10);
        care::graphics::line([(x+25, 145), (x+50, 155), (x+75, 145)], 10);
        care::graphics::line([(x+25, 265), (x+50, 265), (x+75, 240)], 10);
        care::graphics::line([(x+25, 375), (x+75, 375), (x+75, 325)], 10);
        care::graphics::line([(x+25, 425), (x+75, 450), (x+25, 475)], 10);
        care::graphics::line([(x+15, 555), (x+65, 550), (x+15, 545)], 7);
    }
    for x in 1..=6 {
        care::graphics::line_segment((0, x*100), (600, x*100), 2);
        care::graphics::line_segment((x*100, 0), (x*100, 600), 2);
    }
}

care::main!();
