#![warn(missing_docs)]
#![doc = include_str!("../readme.md")]

/// Global care configuration parameters
pub mod config;
/// Low-level event handling
pub mod event;
#[cfg(feature = "graphics")]
/// Contains functions for rendering graphics
pub mod graphics;
/// Stuff for working with a keyboard.
pub mod keyboard;
/// Contains functions for doing various math tasks, including working with vectors
pub mod math;
/// Stuff for working with a mouse
pub mod mouse;
/// Useful structs to have imported
pub mod prelude;
#[cfg(feature = "window")]
/// Contains functions for working with window(s)
pub mod window;

/// Mark a function as the care draw function.
pub use care_macro::care_async_main as async_main;
/// Mark a function as the care draw function.
pub use care_macro::care_draw as draw;
/// Mark a function as the care initialization function.
pub use care_macro::care_init as init;
/// Make some state for the game
pub use care_macro::care_state as state;
/// Mark a function as the care update function.
pub use care_macro::care_update as update;

#[doc(hidden)]
pub use care_macro::care_main as __internal_main;

/// Global care configuration struct, pass to [main] (e.g. `care::main!(Conf { .. })`),
/// to configure the framework
pub use config::Conf;

#[cfg(feature = "graphics")]
/// The image crate is used for loading and saving images from various formats
pub use image;
/// The nalgebra crate is used for vectors and matracies, have fun with math!
pub use nalgebra;
/// The rand crate is used to generate random numbers
pub use rand;
#[cfg(feature = "graphics")]
/// The rust type crate is used internally for loading rendering ttf fonts
pub use rusttype;

/// Inserts a default main function that automatically initializes the framework, opens a window,
/// and calls the functions marked by [init], [update] and [draw] at appropriate
/// times
#[macro_export]
macro_rules! main {
    () => {
        $crate::main!($crate::Conf::default());
    };
    ($conf:expr) => {
        $crate::__internal_main!($conf);
    };
}
