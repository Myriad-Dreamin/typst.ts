use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use std::collections::HashMap;
use std::sync::Arc;

use ttf_parser::gsub::{Ligature, SubstitutionSubtable};
use ttf_parser::GlyphId;
use typst::text::Font;

use reflexo::ImmutStr;

/// get reverse cmap
fn get_rev_cmap(face: &ttf_parser::Face<'_>) -> std::collections::HashMap<GlyphId, char> {
    let mut rev_cmap = std::collections::HashMap::new();
    if let Some(cmap) = face.tables().cmap {
        for subtable in cmap.subtables {
            if !subtable.is_unicode() {
                continue;
            }

            subtable.codepoints(|c| {
                let c = char::from_u32(c);
                if let Some(c) = c {
                    let g = face.glyph_index(c);
                    if let Some(g) = g {
                        rev_cmap.insert(g, c);
                    }
                }
            })
        }
    }
    rev_cmap
}

/// Some ttf fonts do not have well gdbf table, so we need to manually get
/// ligature coverage
fn get_liga_cov(face: &ttf_parser::Face<'_>) -> std::collections::HashSet<GlyphId> {
    let mut res = std::collections::HashSet::new();
    let Some(gsub) = face.tables().gsub else {
        return res;
    };
    for lookup in gsub.lookups {
        for subtable in lookup.subtables.into_iter() {
            if let SubstitutionSubtable::Ligature(ligatures) = subtable {
                for ligature_set in ligatures.ligature_sets {
                    for ligature in ligature_set {
                        res.insert(ligature.glyph);
                    }
                }
            }
        }
    }

    res
}

pub struct LigatureResolver {
    rev_cmap: std::collections::HashMap<GlyphId, char>,
    ligature_cov: std::collections::HashSet<GlyphId>,
    ligature_cmap: RwLock<HashMap<GlyphId, Option<ImmutStr>>>,
}

impl LigatureResolver {
    pub fn new(face: &ttf_parser::Face<'_>) -> Self {
        Self {
            rev_cmap: get_rev_cmap(face),
            ligature_cov: get_liga_cov(face),
            ligature_cmap: RwLock::default(),
        }
    }

    /// resolve the correct unicode string of a ligature glyph
    pub fn resolve(&self, face: &ttf_parser::Face<'_>, id: GlyphId) -> Option<ImmutStr> {
        let cmap = self.ligature_cmap.upgradable_read();
        if let Some(s) = cmap.get(&id) {
            return s.clone();
        }

        let mut cmap = RwLockUpgradableReadGuard::upgrade(cmap);

        let res = if self.ligature_cov.contains(&id) {
            // Combines unicode string from ligature components
            let ligature = LigatureResolver::lookup(face, id).unwrap();

            let c: ImmutStr = ligature
                .components
                .into_iter()
                .map(|g| {
                    self.rev_cmap.get(&g).unwrap_or_else(|| {
                        println!("ligature component not found: {:?} {:?}", g, face);
                        &' '
                    })
                })
                .collect::<String>()
                .into();

            Some(c)
        } else {
            None
        };

        cmap.insert(id, res.clone());
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
}

#[comemo::memoize]
fn get_ligature_resolver(font: &Font) -> Arc<LigatureResolver> {
    Arc::new(LigatureResolver::new(font.ttf()))
}

pub(super) fn resolve_ligature(font: &Font, id: GlyphId) -> Option<ImmutStr> {
    let resolver = get_ligature_resolver(font);
    // let res = resolver.resolve(font.ttf(), id);
    // println!("resolve_ligature {:?} {:?} -> {:?}", font, id, res);
    resolver.resolve(font.ttf(), id)
}
