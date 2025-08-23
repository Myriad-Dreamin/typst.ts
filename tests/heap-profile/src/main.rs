#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let args = std::env::args().collect::<Vec<_>>();
    let action = args[1].as_str();
    let workspace_dir = std::path::Path::new(&args[2]);
    let entry_file_path = std::path::Path::new(&args[3]);

    match action {
        "compile" => {
            typst_ts_heap_profile_test::test_compiler(workspace_dir, entry_file_path);
        }
        _ => panic!("Unknown action: {action}"),
    }
}
