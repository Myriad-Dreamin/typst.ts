use crate::macros::static_assert;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::hash::Hasher;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    ffi::OsStr,
    hash::Hash,
    iter::Peekable,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use typst_ts_core::path::{clean, unix_slash};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct DependentDirInfo {
    pub path: String,
    pub files: DependentFiles,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct DependentFileInfo {
    pub path: PathBuf,
    pub mtime: u64,
}

impl DependentFileInfo {
    pub fn new(path: PathBuf, mtime: u64) -> Self {
        Self { path, mtime }
    }
    pub fn new_ref<P: AsRef<Path>>(path: P, mtime: u64) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            mtime,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct DependentFiles {
    pub children_hash: String,
    pub children: Vec<DependencyTree>,
}

/// A set of files that are known to be dependent on each other.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum DependencyTree {
    Dir(DependentDirInfo),
    Files(DependentFiles),
    File(DependentFileInfo),
}

impl DependencyTree {
    pub fn from_iter(root: &Path, iter: impl Iterator<Item = DependentFileInfo>) -> Self {
        DependencyTreeBuilder::new(root).build(iter)
    }
}

#[derive(Debug, Clone)]
enum InternalInfo {
    Leaf(/* stem */ DependencyTree),
    Stem(/* children */ HashMap<Arc<OsStr>, DirInfoBuilder>),
}

#[derive(Debug, Clone)]
struct DirInfoBuilder {
    path: PathBuf,
    state: InternalInfo,
}

impl PartialEq for DirInfoBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for DirInfoBuilder {}

impl PartialOrd for DirInfoBuilder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl Ord for DirInfoBuilder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

fn collect_files<T: AsRef<[u8]>>(
    path: Option<T>,
    children: impl Iterator<Item = (Vec<u8>, DependencyTree)>,
) -> (Vec<u8>, DependentFiles) {
    let mut built_children = vec![];

    let mut hasher = Sha256::new();
    if let Some(path) = path {
        hasher.update(path.as_ref());
    }
    for (child_hash, child) in children {
        hasher.update(child_hash);
        built_children.push(child);
    }
    let hash = hasher.finalize();
    let children_hash = { format!("sha256:{}", hex::encode(hash)) };
    (
        hash.to_vec(),
        DependentFiles {
            children_hash,
            children: built_children,
        },
    )
}

/// hash is in format sha256:abcdef
/// the children are group in chunk size: 4096/32 = 128
fn build_as_files(
    children: impl Iterator<Item = (Vec<u8>, DependencyTree)>,
) -> (Vec<u8>, DependencyTree) {
    let (hash, files) = collect_files::<Vec<u8>>(None, children);
    (hash, DependencyTree::Files(files))
}

/// hash is in format sha256:abcdef
/// the children are group in chunk size: 4096/32 = 128
fn build_as_dir(
    path: String,
    children: impl ExactSizeIterator<Item = DependencyTree>,
) -> (Vec<u8>, DependencyTree) {
    let (hash, files) = collect_files(Some(&path), make_chunks(children));
    (hash, DependencyTree::Dir(DependentDirInfo { path, files }))
}

fn tuple_second<T, U>((_, b): (T, U)) -> U {
    b
}

enum ChunkBuilder<I: Iterator<Item = DependencyTree>, J: Iterator<Item = Vec<DependencyTree>>> {
    Simple(Option<I>),
    Complex(J),
}

impl<I: Iterator<Item = DependencyTree>, J: Iterator<Item = Vec<DependencyTree>>> Iterator
    for ChunkBuilder<I, J>
{
    type Item = (Vec<u8>, DependencyTree);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Simple(i) => i.take().map(|e| build_as_files(e.map(build_one))),
            Self::Complex(j) => j
                .next()
                .map(|e| build_as_files(e.into_iter().map(build_one))),
        }
    }
}

fn make_chunks(
    iter: impl ExactSizeIterator<Item = DependencyTree>,
) -> impl Iterator<Item = (Vec<u8>, DependencyTree)> {
    if iter.len() <= 128 {
        return ChunkBuilder::Simple(Some(iter));
    }

    const CHUNK_BUCKET: usize = 32;
    static_assert!(CHUNK_BUCKET & (CHUNK_BUCKET - 1) == 0);
    const EMPTY_VEC: Vec<DependencyTree> = Vec::new();

    let mut res: [Vec<DependencyTree>; CHUNK_BUCKET] = [EMPTY_VEC; CHUNK_BUCKET];
    for node in iter {
        match node {
            DependencyTree::Dir(d) => {
                let mut path_hash = DefaultHasher::new();
                d.path.hash(&mut path_hash);
                let path_hash = path_hash.finish() as usize;
                res[path_hash & (CHUNK_BUCKET - 1)].push(DependencyTree::Dir(d));
            }
            DependencyTree::File(d) => {
                let mut path_hash = DefaultHasher::new();
                d.path.hash(&mut path_hash);
                let path_hash = path_hash.finish() as usize;
                res[path_hash & (CHUNK_BUCKET - 1)].push(DependencyTree::File(d));
            }
            _ => unreachable!(),
        }
    }
    ChunkBuilder::Complex(res.into_iter())
}

fn build_one(node: DependencyTree) -> (Vec<u8>, DependencyTree) {
    match node {
        DependencyTree::Dir(info) => {
            let children = info.files.children.into_iter();
            build_as_dir(info.path, children)
        }
        DependencyTree::Files(info) => {
            let children = info.children.into_iter();
            build_as_files(make_chunks(children))
        }
        DependencyTree::File(info) => {
            let mut hasher = Sha256::new();
            hasher.update(info.path.as_os_str().to_string_lossy().as_bytes());
            hasher.update(info.mtime.to_le_bytes());
            (hasher.finalize().to_vec(), DependencyTree::File(info))
        }
    }
}

impl DirInfoBuilder {
    fn new_dir(path: PathBuf) -> Self {
        Self {
            path,
            state: InternalInfo::Stem(HashMap::new()),
        }
    }

    fn new_leaf(path: PathBuf, internal: DependencyTree) -> Self {
        Self {
            path,
            state: InternalInfo::Leaf(internal),
        }
    }

    fn build(self) -> (Vec<u8>, DependencyTree) {
        match self.state {
            InternalInfo::Leaf(node) => build_one(node),
            InternalInfo::Stem(children) => {
                let mut children: Vec<_> = children.into_iter().map(tuple_second).collect();
                children.sort();
                build_as_dir(
                    unix_slash(&self.path),
                    children
                        .into_iter()
                        .map(DirInfoBuilder::build)
                        .map(tuple_second),
                )
            }
        }
    }
}

struct DependencyTreeBuilder {
    root: Option<DirInfoBuilder>,
    prefix: HashMap<Arc<OsStr>, DirInfoBuilder>,
}

impl DependencyTreeBuilder {
    fn new(root: &Path) -> Self {
        Self {
            root: Some(DirInfoBuilder::new_dir(clean(root))),
            prefix: HashMap::new(),
        }
    }

    fn build(mut self, iter: impl Iterator<Item = DependentFileInfo>) -> DependencyTree {
        let mut root = self.root.take().unwrap();
        for mut item in iter {
            // fails on item.path is not absolute
            item.path = pathdiff::diff_paths(&item.path, &root.path).unwrap();

            let mut item_root = &mut root;
            let mut components = item.path.components();
            match item.path.components().peekable().peek() {
                Some(Component::Prefix(seg)) => {
                    let child = self.prefix.get_mut(seg.as_os_str());
                    item_root = match child {
                        Some(child) => child,
                        None => {
                            let child = DirInfoBuilder::new_dir(
                                vec![Component::Prefix(*seg)].into_iter().collect(),
                            );
                            self.prefix.insert(seg.as_os_str().into(), child);
                            self.prefix.get_mut(seg.as_os_str()).unwrap()
                        }
                    };
                    components.next();
                }
                Some(Component::RootDir) => {
                    let rt = OsStr::new("/");
                    let child = self.prefix.get_mut(rt);
                    item_root = match child {
                        Some(child) => child,
                        None => {
                            let child = DirInfoBuilder::new_dir(
                                vec![Component::RootDir].into_iter().collect(),
                            );
                            self.prefix.insert(rt.into(), child);
                            self.prefix.get_mut(rt).unwrap()
                        }
                    };
                    components.next();
                }
                _ => {}
            }

            Self::simulate_touch(item_root, components.peekable(), item.mtime);
        }

        if self.prefix.is_empty() {
            return root.build().1;
        }

        let mut roots = vec![root];
        roots.extend(self.prefix.into_iter().map(tuple_second));
        roots.sort();

        let files = build_as_files(roots.into_iter().map(DirInfoBuilder::build));
        files.1
    }

    // todo: how to perform not recursive approach?
    fn simulate_touch<'a, 'b>(
        node: &'a mut DirInfoBuilder,
        mut leaf_path: Peekable<impl Iterator<Item = Component<'b>>>,
        mtime: u64,
    ) {
        fn simulate_touch_seg<'a, 'b>(
            node: &'a mut DirInfoBuilder,
            seg: &OsStr,
            comp: Component,
            mut leaf_path: Peekable<impl Iterator<Item = Component<'b>>>,
            mtime: u64,
        ) {
            let children = match &mut node.state {
                InternalInfo::Stem(stem) => stem,
                _ => unreachable!(),
            };

            let child = children.get_mut(seg);
            if leaf_path.peek().is_none() {
                let leaf = DependencyTree::File(DependentFileInfo {
                    path: vec![comp].into_iter().collect(),
                    mtime,
                });
                match child {
                    Some(builder) => {
                        builder.state = InternalInfo::Leaf(leaf);
                    }
                    None => {
                        let child =
                            DirInfoBuilder::new_leaf(vec![comp].into_iter().collect(), leaf);
                        children.insert(seg.into(), child);
                    }
                };
                return;
            };

            let child = match child {
                Some(child) => child,
                None => {
                    let child = DirInfoBuilder::new_dir(vec![comp].into_iter().collect());
                    children.insert(seg.into(), child);
                    children.get_mut(seg).unwrap()
                }
            };

            DependencyTreeBuilder::simulate_touch(child, leaf_path, mtime);
        }

        'unwrap_touch: while let Some(comp) = leaf_path.next() {
            match comp {
                std::path::Component::Normal(seg) => {
                    simulate_touch_seg(node, seg, comp, leaf_path, mtime);
                    return;
                }
                std::path::Component::CurDir => continue 'unwrap_touch,
                std::path::Component::ParentDir => {
                    simulate_touch_seg(node, OsStr::new(".."), comp, leaf_path, mtime);
                    return;
                }
                std::path::Component::Prefix(_) => unreachable!(),
                std::path::Component::RootDir => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{DependencyTree, DependentFileInfo};

    #[test]
    fn test_build_dependent_info() {
        let files = [
            DependentFileInfo::new_ref("/test/main.typ", 0xfff),
            DependentFileInfo::new_ref("/test/ref.typ", 0xfff),
        ];
        let tree = DependencyTree::from_iter(Path::new("/test"), files.into_iter());

        let files = [
            DependentFileInfo::new_ref("/test/ref.typ", 0xfff),
            DependentFileInfo::new_ref("/test/main.typ", 0xfff),
        ];
        let tree2 = DependencyTree::from_iter(Path::new("/test"), files.into_iter());

        assert_eq!(tree, tree2);

        let mut files = vec![];
        for i in 0..4096 {
            files.push(DependentFileInfo::new_ref(
                format!("/test/{}.typ", i),
                0xfff,
            ));
        }
        let tree = DependencyTree::from_iter(Path::new("/test"), files.into_iter());

        let mut files = vec![];
        for i in 0..4096 {
            files.push(DependentFileInfo::new_ref(
                format!("/test/{}.typ", i),
                0xfff,
            ));
        }
        let tree2 = DependencyTree::from_iter(Path::new("/test"), files.into_iter());

        assert_eq!(tree, tree2);

        println!("{}", serde_json::to_string_pretty(&tree).unwrap());
    }
}
