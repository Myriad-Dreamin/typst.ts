use std::rc::Rc;

use ttf_parser::gdef::GlyphClass;
use ttf_parser::gsub::{Ligature, SubstitutionSubtable};
use ttf_parser::GlyphId;
use typst::font::Font;

use crate::ImmutStr;

fn is_ligature(face: &ttf_parser::Face<'_>, id: GlyphId) -> bool {
    let table = match face.tables().gdef {
        Some(v) => v,
        None => return false,
    };

    matches!(table.glyph_class(id), Some(GlyphClass::Ligature))
}

/// get reverse cmap
fn get_rev_cmap(face: &ttf_parser::Face<'_>) -> std::collections::HashMap<GlyphId, char> {
    let mut rev_cmap = std::collections::HashMap::new();
    for i in 0..(u16::MAX as usize) {
        let c = char::from_u32(i as u32);
        if let Some(c) = c {
            let g = face.glyph_index(c);
            if let Some(g) = g {
                rev_cmap.insert(g, c);
            }
        }
    }
    rev_cmap
}

pub struct LigatureResolver {
    rev_cmap: std::collections::HashMap<GlyphId, char>,
    ligature_cmap: elsa::FrozenBTreeMap<GlyphId, Box<Option<ImmutStr>>>,
}

impl LigatureResolver {
    pub fn new(face: &ttf_parser::Face<'_>) -> Self {
        Self {
            rev_cmap: get_rev_cmap(face),
            ligature_cmap: elsa::FrozenBTreeMap::new(),
        }
    }

    /// resolve the correct unicode string of a ligature glyph
    pub fn resolve(&self, face: &ttf_parser::Face<'_>, id: GlyphId) -> Option<ImmutStr> {
        if let Some(s) = self.ligature_cmap.get(&id) {
            return s.clone();
        }

        if is_ligature(face, id) {
            return Some(self.get(face, id));
        }

        self.ligature_cmap.insert(id, Box::new(None));
        None
    }

    /// return a combined unicode string from ligature components
    fn get(&self, face: &ttf_parser::Face<'_>, id: GlyphId) -> ImmutStr {
        let ligature = LigatureResolver::lookup(face, id).unwrap();

        let c: ImmutStr = ligature
            .components
            .into_iter()
            .map(|g| self.rev_cmap.get(&g).unwrap())
            .collect::<String>()
            .into();

        self.ligature_cmap.insert(id, Box::new(Some(c.clone())));

        c
    }

    /// lookup ligature without cache
    fn lookup<'a>(face: &ttf_parser::Face<'a>, id: GlyphId) -> Option<Ligature<'a>> {
        let gsub = face.tables().gsub.unwrap();
        for lookup in gsub.lookups {
            for subtable in lookup.subtables.into_iter() {
                if let SubstitutionSubtable::Ligature(ligatures) = subtable {
                    for ligature_set in ligatures.ligature_sets {
                        for ligature in ligature_set {
                            if ligature.glyph == id {
                                return Some(ligature);
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

#[comemo::memoize]
fn get_ligature_resolver(font: &Font) -> Rc<LigatureResolver> {
    Rc::new(LigatureResolver::new(font.ttf()))
}

pub(super) fn resolve_ligature(font: &Font, id: GlyphId) -> Option<ImmutStr> {
    let resolver = get_ligature_resolver(font);
    // println!("resolve_ligature {:?} {:?}", font, id);
    resolver.resolve(font.ttf(), id)
}
