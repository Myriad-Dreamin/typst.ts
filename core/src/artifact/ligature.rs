use ttf_parser::gdef::GlyphClass;
use ttf_parser::gsub::{Ligature, SubstitutionSubtable};
use ttf_parser::GlyphId;
use typst::doc::{Glyph, TextItem};

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
    ligature_cmap: std::collections::HashMap<GlyphId, Option<String>>,
}

impl LigatureResolver {
    pub fn new(face: &ttf_parser::Face<'_>) -> Self {
        Self {
            rev_cmap: get_rev_cmap(face),
            ligature_cmap: std::collections::HashMap::new(),
        }
    }

    /// resolve the correct unicode string of a ligature glyph
    pub fn resolve(
        &mut self,
        face: &ttf_parser::Face<'_>,
        text_context: &TextItem,
        glyph: &Glyph,
    ) -> Option<String> {
        let id = GlyphId(glyph.id);
        if let Some(s) = self.ligature_cmap.get(&id) {
            return s.clone();
        }

        if is_ligature(face, id) {
            return Some(self.get(face, text_context, glyph));
        }

        self.ligature_cmap.insert(id, None);
        None
    }

    /// return a list of covered ligatures
    pub fn into_covered(self) -> Vec<(u16, String)> {
        let mut res = vec![];
        for (k, v) in self.ligature_cmap {
            if let Some(v) = v {
                res.push((k.0, v));
            }
        }
        res
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

    /// return a combined unicode string from ligature components
    fn get(
        &mut self,
        face: &ttf_parser::Face<'_>,
        text_context: &TextItem,
        glyph: &Glyph,
    ) -> String {
        let id = GlyphId(glyph.id);

        let ligature = LigatureResolver::lookup(face, id).unwrap();

        // todo: this does not works on emoji
        let c = ligature
            .components
            .into_iter()
            .map(|g| self.rev_cmap.get(&g).unwrap())
            .collect::<String>();

        let c = text_context.text[glyph.range.start as usize..glyph.range.start as usize]
            .to_string()
            + c.as_str();
        self.ligature_cmap.insert(id, Some(c.clone()));

        c
    }
}
