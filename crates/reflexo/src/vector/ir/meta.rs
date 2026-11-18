use super::preludes::*;

/// Item representing an `<a/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct LinkItem {
    /// The target of the link item.
    pub href: ImmutStr,
    /// The box size of the link item.
    pub size: Size,
}

impl LinkItem {
    /// Parse the renderer's internal page-location link without evaluating its target.
    pub fn typst_location(&self) -> Option<(usize, f32, f32)> {
        let location = self
            .href
            .strip_prefix("@typst:handleTypstLocation(this, ")?;
        let (page, coordinates) = location.split_once(", ")?;
        let (x, y) = coordinates.split_once(", ")?;
        let y = y.strip_suffix(')')?;
        let (page, x, y) = (
            page.parse().ok()?,
            x.parse::<f32>().ok()?,
            y.parse::<f32>().ok()?,
        );

        (x.is_finite() && y.is_finite()).then_some((page, x, y))
    }
}

/// Source mapping from vec item to source span.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum SourceMappingNode {
    Group(Arc<[u64]>),
    Text(SpanId),
    Image(SpanId),
    Shape(SpanId),
    Page(u64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::ir::{Axes, Scalar};

    fn link(href: &str) -> LinkItem {
        LinkItem {
            href: href.into(),
            size: Axes::new(Scalar(1.0), Scalar(1.0)),
        }
    }

    #[test]
    fn typst_location_accepts_only_finite_generated_coordinates() {
        assert_eq!(
            link("@typst:handleTypstLocation(this, 2, 1.5, 3.25)").typst_location(),
            Some((2, 1.5, 3.25)),
        );
        for href in [
            "@typst:custom-route",
            "@typst:handleTypstLocation(this, 2, inf, 3.25)",
            "@typst:handleTypstLocation(this, 2, 1.5, 3.25); custom-route",
        ] {
            assert!(link(href).typst_location().is_none(), "{href}");
        }
    }
}
