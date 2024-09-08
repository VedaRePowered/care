# Care
Care is a very simple game framework focused solely on ease of use. The api is heavilty inspired by [Löve 2D](https://love2d.org/).

## Hello World

Here's the source code for hello world with care, making a window appear that displays "hello world" (using the love2d-inspired interface).

```rs
#[care::draw]
fn draw() {
    care::graphics::text("Hello, World!", (20, 20));
}

care::main!();
```

You can also use the async interface (inspired by macroquad) if you want:
```rs
#[care::async_main]
async fn main() {
    loop {
        care::graphics::text("Hello, World!", (20, 20));

        care::event::next_frame().await;
    }
}

care::main!();
```

See examples in the examples directory or the rust doc for more information.

## Comparison with love2d

### Graphics
The namespace `love.graphics` and `care::graphics`.
- [ ] Drawing
    - [ ] `love.graphics.arc`
    - [x] `love.graphics.circle`
    - [ ] `love.graphics.clear`
    - [ ] `love.graphics.discard`
    - [ ] `love.graphics.draw` -> partial: `texture`, `texture_scale`, `texture_source`, `texture_rot` & `texture_rounded`
    - [ ] `love.graphics.drawInstanced`
    - [ ] `love.graphics.drawLayer`
    - [x] `love.graphics.ellipse`
    - [ ] `love.graphics.flushBatch`
    - [x] `love.graphics.line` -> `line_segment`, `line` & `line_varying_styles`
    - [ ] `love.graphics.points`
    - [ ] `love.graphics.polygon` -> partial: `line` & `line_varying_styles`
    - [x] `love.graphics.present`
    - [ ] `love.graphics.print` -> partial: `text`
    - [ ] `love.graphics.printf`
    - [x] `love.graphics.rectangle` -> `rectangle`, `rectangle_rot` & `rectangle_rounded`
    - [ ] `love.graphics.stencil`
- [ ] Object Creation
    - [ ] `love.graphics.newImage` -> `Texture::new`, `new_from_file_format`, `new_fill`, `new_from_image` & `new_from_data`
    - [ ] `love.graphics.newFont` -> `Font::new`, `Font::new_from_vec` & `Font::new_from_bytes`
    - [ ] Many more...
- [ ] Graphics State
    - [x] `love.graphics.setColor` -> `set_colour`
    - [x] `love.graphics.setLineJoin` -> `set_line`
    - [x] `love.graphics.setLineWidth` -> Not needed
    - [ ] `love.graphics.setLineStyle`
    - [ ] Many more...
- [ ] Coordinate System
- [ ] Window
- [ ] System Information

### Keyboard
The namespace `love.keyboard` and `care::keyboard`.
- [x] `love.keyboard.isDown` -> `is_down`
- [ ] `love.keyboard.getKeyFromScancode`
- [ ] `love.keyboard.getScancodeFromKey`
- [ ] `love.keyboard.hasKeyRepeat`
- [ ] `love.keyboard.hasScreenKeyboard`
- [ ] `love.keyboard.hasTextInput`
- [ ] `love.keyboard.isScancodeDown`
- [ ] `love.keyboard.setKeyRepeat`
- [ ] `love.keyboard.setTextInput`
- Additionally, `is_pressed` & `is_released` are available

