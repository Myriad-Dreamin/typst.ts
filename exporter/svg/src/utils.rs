#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[allow(unused_macros)]
macro_rules! console_log {
    ($($arg:tt)*) => {
        web_sys::console::info_1(&format!(
            $($arg)*
        ).into());
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[allow(unused_macros)]
macro_rules! console_log {
    ($($arg:tt)*) => {
        println!(
            $($arg)*
        );
    }
}

#[allow(unused_imports)]
pub(crate) use console_log;
use tiny_skia::Transform;
use typst::geom::{Abs, Color};

/// Additional methods for [`Length`].
pub trait AbsExt {
    /// Convert to a number of points as f32.
    fn to_f32(self) -> f32;
}

impl AbsExt for Abs {
    fn to_f32(self) -> f32 {
        self.to_pt() as f32
    }
}

pub trait ToCssExt {
    fn to_css(self) -> String;
}

impl ToCssExt for Color {
    fn to_css(self) -> String {
        let color = self.to_rgba();
        if color.a == 255 {
            let shorter = format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b);
            if shorter.chars().nth(1) == shorter.chars().nth(2)
                && shorter.chars().nth(3) == shorter.chars().nth(4)
                && shorter.chars().nth(5) == shorter.chars().nth(6)
            {
                return format!(
                    "#{}{}{}",
                    shorter.chars().nth(1).unwrap(),
                    shorter.chars().nth(3).unwrap(),
                    shorter.chars().nth(5).unwrap()
                );
            }
            return shorter;
        }

        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            color.r, color.g, color.b, color.a
        )
    }
}

impl ToCssExt for Transform {
    fn to_css(self) -> String {
        format!(
            r#"matrix({},{},{},{},{},{})"#,
            self.sx, self.ky, self.kx, self.sy, self.tx, self.ty
        )
    }
}

pub struct PerfEvent {}
