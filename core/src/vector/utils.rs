use tiny_skia::Transform;
use typst::layout::Abs;
use typst::visualize::{Color, Paint};

use super::ir;

/// Additional methods for [`typst::layout::Abs`].
pub trait AbsExt {
    /// Convert to a number of points as f32.
    fn to_f32(self) -> f32;
}

impl AbsExt for Abs {
    fn to_f32(self) -> f32 {
        self.to_pt() as f32
    }
}

/// Additional methods for types that can be converted to CSS.
pub trait ToCssExt {
    fn to_css(self) -> String;
}

impl ToCssExt for Color {
    fn to_css(self) -> String {
        let [r, g, b, a] = self.to_vec4_u8();
        if a == 255 {
            let shorter = format!("#{:02x}{:02x}{:02x}", r, g, b);
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

        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
    }
}

impl ToCssExt for Paint {
    fn to_css(self) -> String {
        let fill = match self {
            Paint::Solid(color) => color,
            // todo: pattern
            Paint::Pattern(..) => Color::BLACK,
            // todo: gradient
            Paint::Gradient(..) => Color::BLACK,
        };
        fill.to_css()
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

impl ToCssExt for ir::Transform {
    fn to_css(self) -> String {
        format!(
            r#"matrix({},{},{},{},{},{})"#,
            self.sx.0, self.ky.0, self.kx.0, self.sy.0, self.tx.0, self.ty.0
        )
    }
}
