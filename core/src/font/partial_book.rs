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

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for (idx, info, slot) in &self.changes {
            s.push_str(&format!(
                "{:?}: {} -> {:?}\n",
                idx,
                info.family,
                slot.get_uninitialized()
            ));
        }
        s
    }
}
