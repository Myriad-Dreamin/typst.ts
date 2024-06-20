// use std::{collections::HashMap, ffi::OsStr, path::Path, sync::Arc};

// use parking_lot::{RwLock, RwLockUpgradableReadGuard};
// use reflexo::ImmutPath;
// use typst::diag::{FileError, FileResult};

// use crate::{AccessModel, Bytes, QueryRef, Time};

// /// incrementally query a value from a self holding state
// type IncrQueryRef<S, E> = QueryRef<S, E, Option<S>>;

// /// Holds the cached data of a single file
// #[derive(Debug)]
// pub struct CacheEntry<S> {
//     /// The last lifetime count when the cache is updated
//     last_access_lifetime: usize,
//     /// The cached mtime of the file
//     mtime: Time,
//     /// Whether the file is a file, lazily triggered when mtime is changed
//     is_file: QueryRef<bool, FileError>,
//     /// The content of the file, lazily triggered when mtime is changed
//     read_all: QueryRef<Bytes, FileError>,
//     /// The incremental state of the source, lazily triggered when mtime is
//     /// changed
//     source_state: IncrQueryRef<S, FileError>,
// }

// /// Provides general cache to file access.
// #[derive(Debug)]
// pub struct CachedAccessModel<Inner: AccessModel, C> {
//     /// The underlying access model for real file access
//     inner: Inner,
//     /// The lifetime count which resembles [`crate::vfs::Vfs::lifetime_cnt`]
//     ///
//     /// Note: The lifetime counter is incremented on resetting vfs.
//     lifetime_cnt: usize,
//     /// The cache entries for each paths
//     cache_entries: RwLock<HashMap<Arc<OsStr>, CacheEntry<C>>>,
// }

// impl<Inner: AccessModel, C> CachedAccessModel<Inner, C> {
//     /// Create a new [`CachedAccessModel`] with the given inner access model
//     pub fn new(inner: Inner) -> Self {
//         CachedAccessModel {
//             inner,
//             lifetime_cnt: 1,
//             cache_entries: RwLock::new(HashMap::new()),
//         }
//     }

//     /// Get the inner access model
//     pub fn inner(&self) -> &Inner {
//         &self.inner
//     }

//     /// Get the mutable reference to the inner access model
//     pub fn inner_mut(&mut self) -> &mut Inner {
//         &mut self.inner
//     }
// }

// impl<Inner: AccessModel, C: Clone> CachedAccessModel<Inner, C> {
//     fn mtime_inner(&self, src: &Path) -> FileResult<Time> {
//         self.inner.mtime(src)
//     }

//     fn cache_entry<T>(
//         &self,
//         src: &Path,
//         cb: impl FnOnce(&CacheEntry<C>) -> FileResult<T>,
//     ) -> FileResult<T> {
//         let path_key = src.as_os_str();
//         let path_results = self.cache_entries.upgradable_read();
//         let entry = path_results.get(path_key);
//         let (new_mtime, prev_to_diff) = if let Some(entry) = entry {
//             if entry.last_access_lifetime == self.lifetime_cnt {
//                 return cb(entry);
//             }

//             let mtime = self.mtime_inner(src)?;
//             if mtime == entry.mtime {
//                 return cb(entry);
//             }

//             (
//                 mtime,
//                 entry
//                     .source_state
//                     .get_uninitialized()
//                     .and_then(|e| e.clone().ok()),
//             )
//         } else {
//             (self.mtime_inner(src)?, None)
//         };

//         let mut path_results = RwLockUpgradableReadGuard::upgrade(path_results);

//         path_results.insert(
//             path_key.into(),
//             CacheEntry {
//                 last_access_lifetime: self.lifetime_cnt,
//                 mtime: new_mtime,
//                 is_file: QueryRef::default(),
//                 read_all: QueryRef::default(),
//                 source_state: QueryRef::with_context(prev_to_diff),
//             },
//         );

//         drop(path_results);
//         let path_results = self.cache_entries.read();
//         cb(path_results.get(path_key).unwrap())
//     }
// }

// impl<Inner: AccessModel, C: Clone> CachedAccessModel<Inner, C> {
//     /// This is not a common interface for access model, but it is used for vfs
//     /// incremental parsing.
//     pub fn read_all_diff(
//         &self,
//         src: &Path,
//         compute: impl FnOnce(Option<C>, String) -> FileResult<C>,
//     ) -> FileResult<C> {
//         self.cache_entry(src, |entry| {
//             let data = entry.source_state.compute_with_context(|prev_to_diff| {
//                 let data = entry.read_all.compute(|| self.inner.content(src))?;
//                 let text = from_utf8_or_bom(data)?.to_owned();
//                 compute(prev_to_diff, text)
//             })?;

//             let t = data.clone();
//             Ok(t)
//         })
//     }
// }

// impl<Inner: AccessModel, C: Clone> AccessModel for CachedAccessModel<Inner, C> {
//     fn clear(&mut self) {
//         self.lifetime_cnt += 1;

//         let mut path_results = self.cache_entries.write();
//         let new_lifetime = self.lifetime_cnt;
//         path_results.retain(|_, v| new_lifetime - v.last_access_lifetime <= 30);
//     }

//     fn mtime(&self, src: &Path) -> FileResult<Time> {
//         self.cache_entry(src, |entry| Ok(entry.mtime))
//     }

//     fn is_file(&self, src: &Path) -> FileResult<bool> {
//         self.cache_entry(src, |entry| {
//             entry.is_file.compute(|| self.inner.is_file(src)).copied()
//         })
//     }

//     fn real_path(&self, src: &Path) -> FileResult<ImmutPath> {
//         // todo: cache real path
//         self.inner.real_path(src)
//     }

//     fn content(&self, src: &Path) -> FileResult<Bytes> {
//         self.cache_entry(src, |entry| {
//             let data = entry.read_all.compute(|| self.inner.content(src));
//             Ok(data?.clone())
//         })
//     }
// }

// use std::{path::PathBuf, sync::Mutex, time::Duration};

// use crossbeam::channel::{unbounded, Sender};
// use dashmap::{mapref::entry::Entry, DashMap};
// use eyre::{eyre, Context, Report, Result};
// use notify_debouncer_mini::{
//     new_debouncer,
//     notify::{RecommendedWatcher, RecursiveMode},
//     DebounceEventResult, Debouncer,
// };
// use salsa::DebugWithDb;

// // ANCHOR: main
// fn main() -> Result<()> {
//     // Create the channel to receive file change events.
//     let (tx, rx) = unbounded();
//     let mut db = Database::new(tx);

//     let initial_file_path = std::env::args_os()
//         .nth(1)
//         .ok_or_else(|| eyre!("Usage: ./lazy-input <input-file>"))?;

//     // Create the initial input using the input method so that changes to it
//     // will be watched like the other files.
//     let initial = db.input(initial_file_path.into())?;
//     loop {
//         // Compile the code starting at the provided input, this will read other
//         // needed files using the on-demand mechanism.
//         let sum = compile(&db, initial);
//         let diagnostics = compile::accumulated::<Diagnostic>(&db, initial);
//         if diagnostics.is_empty() {
//             println!("Sum is: {}", sum);
//         } else {
//             for diagnostic in diagnostics {
//                 println!("{}", diagnostic);
//             }
//         }

//         for log in db.logs.lock().unwrap().drain(..) {
//             eprintln!("{}", log);
//         }

//         // Wait for file change events, the output can't change unless the
//         // inputs change.
//         for event in rx.recv()?.unwrap() {
//             let path = event.path.canonicalize().wrap_err_with(|| {
//                 format!("Failed to canonicalize path {}", event.path.display())
//             })?;
//             let file = match db.files.get(&path) {
//                 Some(file) => *file,
//                 None => continue,
//             };
//             // `path` has changed, so read it and update the contents to match.
//             // This creates a new revision and causes the incremental algorithm
//             // to kick in, just like any other update to a salsa input.
//             let contents = std::fs::read_to_string(path)
//                 .wrap_err_with(|| format!("Failed to read file {}", event.path.display()))?;
//             file.set_contents(&mut db).to(contents);
//         }
//     }
// }
// // ANCHOR_END: main

// #[salsa::jar(db = Db)]
// struct Jar(Diagnostic, File, ParsedFile<'_>, compile, parse, sum);

// // ANCHOR: db
// #[salsa::input]
// struct File {
//     path: PathBuf,
//     #[return_ref]
//     contents: String,
// }

// trait Db: salsa::DbWithJar<Jar> {
//     fn input(&self, path: PathBuf) -> Result<File>;
// }

// #[salsa::db(Jar)]
// struct Database {
//     storage: salsa::Storage<Self>,
//     logs: Mutex<Vec<String>>,
//     files: DashMap<PathBuf, File>,
//     file_watcher: Mutex<Debouncer<RecommendedWatcher>>,
// }

// impl Database {
//     fn new(tx: Sender<DebounceEventResult>) -> Self {
//         let storage = Default::default();
//         Self {
//             storage,
//             logs: Default::default(),
//             files: DashMap::new(),
//             file_watcher: Mutex::new(new_debouncer(Duration::from_secs(1), None, tx).unwrap()),
//         }
//     }
// }

// impl Db for Database {
//     fn input(&self, path: PathBuf) -> Result<File> {
//         let path = path
//             .canonicalize()
//             .wrap_err_with(|| format!("Failed to read {}", path.display()))?;
//         Ok(match self.files.entry(path.clone()) {
//             // If the file already exists in our cache then just return it.
//             Entry::Occupied(entry) => *entry.get(),
//             // If we haven't read this file yet set up the watch, read the
//             // contents, store it in the cache, and return it.
//             Entry::Vacant(entry) => {
//                 // Set up the watch before reading the contents to try to avoid
//                 // race conditions.
//                 let watcher = &mut *self.file_watcher.lock().unwrap();
//                 watcher
//                     .watcher()
//                     .watch(&path, RecursiveMode::NonRecursive)
//                     .unwrap();
//                 let contents = std::fs::read_to_string(&path)
//                     .wrap_err_with(|| format!("Failed to read {}", path.display()))?;
//                 *entry.insert(File::new(self, path, contents))
//             }
//         })
//     }
// }
// // ANCHOR_END: db

// impl salsa::Database for Database {
//     fn salsa_event(&self, event: salsa::Event) {
//         // don't log boring events
//         if let salsa::EventKind::WillExecute { .. } = event.kind {
//             self.logs
//                 .lock()
//                 .unwrap()
//                 .push(format!("{:?}", event.debug(self)));
//         }
//     }
// }

// #[salsa::accumulator]
// struct Diagnostic(String);

// impl Diagnostic {
//     fn push_error(db: &dyn Db, file: File, error: Report) {
//         Diagnostic::push(
//             db,
//             format!(
//                 "Error in file {}: {:?}\n",
//                 file.path(db)
//                     .file_name()
//                     .unwrap_or_else(|| "<unknown>".as_ref())
//                     .to_string_lossy(),
//                 error,
//             ),
//         )
//     }
// }

// #[salsa::tracked]
// struct ParsedFile<'db> {
//     value: u32,
//     #[return_ref]
//     links: Vec<ParsedFile<'db>>,
// }

// #[salsa::tracked]
// fn compile(db: &dyn Db, input: File) -> u32 {
//     let parsed = parse(db, input);
//     sum(db, parsed)
// }

// #[salsa::tracked]
// fn parse<'db>(db: &'db dyn Db, input: File) -> ParsedFile<'db> {
//     let mut lines = input.contents(db).lines();
//     let value = match lines.next().map(|line| (line.parse::<u32>(), line)) {
//         Some((Ok(num), _)) => num,
//         Some((Err(e), line)) => {
//             Diagnostic::push_error(
//                 db,
//                 input,
//                 Report::new(e).wrap_err(format!(
//                     "First line ({}) could not be parsed as an integer",
//                     line
//                 )),
//             );
//             0
//         }
//         None => {
//             Diagnostic::push_error(db, input, eyre!("File must contain an integer"));
//             0
//         }
//     };
//     let links = lines
//         .filter_map(|path| {
//             let relative_path = match path.parse::<PathBuf>() {
//                 Ok(path) => path,
//                 Err(err) => {
//                     Diagnostic::push_error(
//                         db,
//                         input,
//                         Report::new(err).wrap_err(format!("Failed to parse path: {}", path)),
//                     );
//                     return None;
//                 }
//             };
//             let link_path = input.path(db).parent().unwrap().join(relative_path);
//             match db.input(link_path) {
//                 Ok(file) => Some(parse(db, file)),
//                 Err(err) => {
//                     Diagnostic::push_error(db, input, err);
//                     None
//                 }
//             }
//         })
//         .collect();
//     ParsedFile::new(db, value, links)
// }

// #[salsa::tracked]
// fn sum<'db>(db: &'db dyn Db, input: ParsedFile<'db>) -> u32 {
//     input.value(db)
//         + input
//             .links(db)
//             .iter()
//             .map(|&file| sum(db, file))
//             .sum::<u32>()
// }
