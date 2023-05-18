use core::fmt;

use typst::font::FontInfo;

use crate::FontSlot;

#[derive(Default)]
pub struct PartialFontBook {
    pub partial_hit: bool,
    pub changes: Vec<(Option<usize>, FontInfo, FontSlot)>,
}

impl PartialFontBook {
    pub fn push(&mut self, change: (Option<usize>, FontInfo, FontSlot)) {
        self.partial_hit = true;
        self.changes.push(change);
    }
}

impl fmt::Display for PartialFontBook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, info, slot) in &self.changes {
            writeln!(
                f,
                "{:?}: {} -> {:?}\n",
                idx,
                info.family,
                slot.get_uninitialized()
            )?;
        }

        Ok(())
    }
}
