use core::fmt;

use typst_ts_core::vector::flat_ir::SourceMappingNode;

pub struct SrcMappingRepr<'a> {
    mapping: &'a [SourceMappingNode],
}

impl fmt::Display for SrcMappingRepr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut st = false;
        for e in self.mapping {
            if st {
                write!(f, "|")?;
            } else {
                st = true;
            }
            match e {
                SourceMappingNode::Page(p) => write!(f, "p,{:x}", p)?,
                SourceMappingNode::Text(t) => write!(f, "t,{:x}", t)?,
                SourceMappingNode::Image(i) => write!(f, "i,{:x}", i)?,
                SourceMappingNode::Shape(s) => write!(f, "s,{:x}", s)?,
                SourceMappingNode::Group(refs) => {
                    f.write_str("g")?;
                    for r in refs.iter() {
                        write!(f, ",{:x}", r)?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn generate_src_mapping(mapping: &[SourceMappingNode]) -> SrcMappingRepr<'_> {
    SrcMappingRepr { mapping }
}
