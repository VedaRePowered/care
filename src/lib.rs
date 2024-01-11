pub mod config;
pub mod graphics;
pub mod math;

// Mark a function as the care initialization function.
pub use care_macro::care_init as init;
// Mark a function as the care update function.
pub use care_macro::care_update as update;
// Mark a function as the care draw function.
pub use care_macro::care_draw as draw;

#[doc(hidden)]
pub use care_macro::care_main as __internal_main;

pub use config::Conf;

/// The nalgebra crate is used for vectors and matracies, have fun with math!
pub use nalgebra;

#[macro_export]
macro_rules! main {
    () => {
        $crate::main!($crate::Conf::default());
    };
    ($conf:expr) => {
        $crate::__internal_main!($conf);
    }
}
