use reflexo::vector::ir::Rgba8Item;
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
    /// Convert to a CSS string.
    fn to_css(self) -> String;
}

impl ToCssExt for Rgba8Item {
    fn to_css(self) -> String {
        let Rgba8Item { r, g, b, a } = self;
        if a == 255 {
            let shorter = format!("#{r:02x}{g:02x}{b:02x}");
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

        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

impl ToCssExt for Color {
    fn to_css(self) -> String {
        let (r, g, b, a) = self.to_rgb().into_format::<u8, u8>().into_components();
        Rgba8Item { r, g, b, a }.to_css()
    }
}

impl ToCssExt for Paint {
    fn to_css(self) -> String {
        let fill = match self {
            Paint::Solid(color) => color,
            // todo: pattern
            Paint::Tiling(..) => Color::BLACK,
            // todo: gradient
            Paint::Gradient(..) => Color::BLACK,
        };
        fill.to_css()
    }
}

impl ToCssExt for ir::Transform {
    fn to_css(self) -> String {
        let regular_scale = self.sx.0 == 1.0 && self.sy.0 == 1.0;
        let regular_skew = self.kx.0 == 0.0 && self.ky.0 == 0.0;
        let regular_translate = self.tx.0 == 0.0 && self.ty.0 == 0.0;

        match (regular_scale, regular_skew, regular_translate) {
            (true, true, true) => String::default(),
            (true, true, false) => format!(r#"translate({},{})"#, self.tx.0, self.ty.0),
            (true, false, true) => format!(r#"skew({},{})"#, self.kx.0, self.ky.0),
            (false, true, true) => format!(r#"scale({},{})"#, self.sx.0, self.sy.0),
            _ => format!(
                r#"matrix({},{},{},{},{},{})"#,
                self.sx.0, self.ky.0, self.kx.0, self.sy.0, self.tx.0, self.ty.0
            ),
        }
    }
}
