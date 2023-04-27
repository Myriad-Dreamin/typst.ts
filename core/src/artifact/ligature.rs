use ttf_parser::gdef::GlyphClass;
use ttf_parser::gsub::{Ligature, SubstitutionSubtable};
use ttf_parser::GlyphId;
use typst::doc::Glyph;

fn is_ligature(face: &ttf_parser::Face<'_>, id: GlyphId) -> bool {
    let table = match face.tables().gdef {
        Some(v) => v,
        None => return false,
    };

    match table.glyph_class(id) {
        Some(GlyphClass::Ligature) => true,
        _ => false,
    }
}

pub struct LigatureResolver {
    rev_cmap: std::collections::HashMap<GlyphId, char>,
    ligature_cmap: std::collections::HashMap<GlyphId, Option<String>>,
}

impl LigatureResolver {
    pub fn new(face: &ttf_parser::Face<'_>) -> Self {
        let mut rev_cmap = std::collections::HashMap::new();
        for i in 0..65535 {
            let c = char::from_u32(i as u32);
            if let Some(c) = c {
                let g = face.glyph_index(c);
                if let Some(g) = g {
                    rev_cmap.insert(g, c);
                }
            }
        }

        Self {
            rev_cmap,
            ligature_cmap: std::collections::HashMap::new(),
        }
    }

    fn lookup<'a>(face: &ttf_parser::Face<'a>, id: GlyphId) -> Option<Ligature<'a>> {
        let gsub = face.tables().gsub.unwrap();
        // let lookup = gsub.lookups.get(id.0).unwrap();
        for lookup in gsub.lookups {
            for subtable in lookup.subtables.into_iter() {
                match subtable {
                    SubstitutionSubtable::Ligature(ligatures) => {
                        for ligature_set in ligatures.ligature_sets {
                            for ligature in ligature_set {
                                if ligature.glyph == id {
                                    return Some(ligature);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        None
    }

    fn get(&mut self, face: &ttf_parser::Face<'_>, glyph: &Glyph) -> String {
        let id = GlyphId(glyph.id);

        let ligature = LigatureResolver::lookup(face, id).unwrap();

        let c = ligature
            .components
            .into_iter()
            .map(|g| self.rev_cmap.get(&g).unwrap())
            .collect::<String>();

        let c = glyph.c.to_string() + c.as_str();
        self.ligature_cmap.insert(id, Some(c.clone()));

        c
    }

    pub fn resolve(&mut self, face: &ttf_parser::Face<'_>, glyph: &Glyph) -> Option<String> {
        let id = GlyphId(glyph.id);
        if let Some(s) = self.ligature_cmap.get(&id) {
            return s.clone();
        }

        if is_ligature(face, id) {
            return Some(self.get(face, glyph));
        }

        self.ligature_cmap.insert(id, None);
        None
    }

    pub fn to_covered(self) -> Vec<(u16, String)> {
        let mut res = vec![];
        for (k, v) in self.ligature_cmap {
            if let Some(v) = v {
                res.push((k.0, v));
            }
        }
        res
    }
}
