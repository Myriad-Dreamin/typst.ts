use std::{
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
};

use clap::{Args, Command, FromArgMatches};
use typst::{doc::Document, font::FontVariant, World};

use typst_ts_cli::{
    compile::compile_export,
    font::EMBEDDED_FONT,
    utils::{self, make_absolute, UnwrapOrExit},
    version::intercept_version,
    CompileArgs, CompileOnceArgs, CompletionArgs, EnvKey, FontSubCommands, GenPackagesDocArgs,
    LinkPackagesArgs, ListFontsArgs, ListPackagesArgs, MeasureFontsArgs, Opts, PackageSubCommands,
    QueryArgs, QueryReplArgs, Subcommands,
};
use typst_ts_compiler::TypstSystemWorld;
use typst_ts_core::exporter_builtins::GroupExporter;
use typst_ts_core::{
    config::CompileOpts,
    exporter_utils::map_err,
    path::{unix_slash, PathClean},
};
fn get_cli(sub_command_required: bool) -> Command {
    let cli = Command::new("$").disable_version_flag(true);
    Opts::augment_args(cli).subcommand_required(sub_command_required)
}

fn help_sub_command() {
    Opts::from_arg_matches(&get_cli(true).get_matches()).unwrap();
}

fn main() {
    human_panic::setup_panic!();

    let opts = Opts::from_arg_matches(&get_cli(false).get_matches())
        .map_err(|err| err.exit())
        .unwrap();

    {
        let mut builder = env_logger::builder();
        builder.filter_level(log::LevelFilter::Info);
        // Better?
        if !matches!(&opts.sub, Some(Subcommands::Compile(CompileArgs { trace: _trace @ Some(_), .. }))) {
            builder
                .filter_module("typst::", log::LevelFilter::Warn)
                .filter_module("typst_library::", log::LevelFilter::Warn);
        }
        builder.init();
    }

    intercept_version(opts.version, opts.vv);

    match opts.sub {
        Some(Subcommands::Compile(args)) => compile(args),
        Some(Subcommands::Query(args)) => query(args),
        Some(Subcommands::QueryRepl(args)) => query_repl(args),
        Some(Subcommands::Completion(args)) => generate_completion(args),
        Some(Subcommands::Env(args)) => match args.key {
            EnvKey::Features => {
                intercept_version(false, typst_ts_cli::version::VersionFormat::Features)
            }
        },
        Some(Subcommands::Font(font_sub)) => match font_sub {
            FontSubCommands::List(args) => list_fonts(args),
            FontSubCommands::Measure(args) => measure_fonts(args),
        },
        Some(Subcommands::Package(pkg_sub)) => match pkg_sub {
            PackageSubCommands::List(args) => list_packages(args),
            PackageSubCommands::Link(args) => link_packages(args, false),
            PackageSubCommands::Unlink(args) => link_packages(args, true),
            PackageSubCommands::Doc(args) => doc_packages(args),
        },
        None => help_sub_command(),
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn compile(args: CompileArgs) -> ! {
    let entry_file_path = Path::new(args.compile.entry.as_str()).clean();
    let exporter = typst_ts_cli::export::prepare_exporters(&args, &entry_file_path);

    compile_export(args, exporter)
}

/// Execute a query command.
pub fn query(args: QueryArgs) -> ! {
    use typst_ts_cli::query::format;
    use typst_ts_compiler::service::query::retrieve;
    let compile_args = args.compile.clone();

    let mut exporter = GroupExporter::<Document>::new(vec![]);

    exporter.push_front(Box::new(move |world: &dyn World, output: Arc<Document>| {
        let data = retrieve(world, &args.selector, &output).map_err(map_err)?;
        let serialized = format(data, &args).map_err(map_err)?;
        println!("{serialized}");
        Ok(())
    }));

    compile_export(compile_args, exporter)
}

fn query_repl(args: QueryReplArgs) -> ! {
    use typst_ts_cli::query_repl::start_repl_test;
    let compile_args = args.compile.clone();

    start_repl_test(compile_args).unwrap();
    exit(0)
}

fn generate_completion(CompletionArgs { shell }: CompletionArgs) -> ! {
    clap_complete::generate(
        shell,
        &mut get_cli(true),
        "typst-ts-cli",
        &mut std::io::stdout(),
    );
    exit(0);
}

fn list_fonts(command: ListFontsArgs) -> ! {
    let mut root_path = PathBuf::new();
    // todo: should cover default workspace path
    root_path.push("-");

    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: root_path,
        font_paths: command.font_paths,
        with_embedded_fonts: EMBEDDED_FONT.to_owned(),
        ..CompileOpts::default()
    })
    .unwrap_or_exit();

    for (name, infos) in world.book().families() {
        println!("{name}");
        if command.variants {
            for info in infos {
                let FontVariant {
                    style,
                    weight,
                    stretch,
                } = info.variant;
                println!("- Style: {style:?}, Weight: {weight:?}, Stretch: {stretch:?}");
            }
        }
    }

    exit(0)
}

fn measure_fonts(args: MeasureFontsArgs) -> ! {
    let mut root_path = PathBuf::new();
    // todo: should cover default workspace path
    root_path.push("-");

    let mut font_profile_paths = vec![];
    if args.output.exists() {
        font_profile_paths.push(args.output.clone());
    }

    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: root_path,
        font_profile_paths,
        font_paths: args.font_paths,
        font_profile_cache_path: args.output.clone(),
        no_system_fonts: args.no_system_fonts,
        ..CompileOpts::default()
    })
    .unwrap_or_exit();

    // create directory for args.output
    if let Some(output) = args.output.parent() {
        std::fs::create_dir_all(output).unwrap();
    }

    let profile = serde_json::to_vec(world.font_resolver.profile()).unwrap();

    // gzip
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&profile).unwrap();
    std::fs::write(args.output, encoder.finish().unwrap()).unwrap();

    exit(0)
}

fn list_packages(args: ListPackagesArgs) -> ! {
    fn get_string(v: &toml::Value) -> &str {
        match v {
            toml::Value::String(table) => table,
            _ => unreachable!(),
        }
    }

    let world = TypstSystemWorld::new(CompileOpts::default()).unwrap_or_exit();

    let paths = world.registry.paths();

    for dir in paths {
        let dir_pretty = unix_slash(&dir);
        let namespaces = std::fs::read_dir(dir).unwrap();

        for ns in namespaces {
            let ns = ns.unwrap();
            let ns_pretty = ns.file_name();
            let ns_pretty = ns_pretty.to_string_lossy();

            let packages = std::fs::read_dir(ns.path()).unwrap();
            for pkg_base in packages {
                let packages2 = std::fs::read_dir(pkg_base.unwrap().path()).unwrap();
                for pkg in packages2 {
                    let pkg = pkg.unwrap();
                    let manifest_path = pkg.path().join("typst.toml");
                    let manifest = std::fs::read_to_string(manifest_path).unwrap();
                    let manifest: toml::Table = toml::from_str(&manifest).unwrap();

                    let pkg_info = match manifest.get("package").unwrap() {
                        toml::Value::Table(table) => table,
                        _ => unreachable!(),
                    };

                    let name = get_string(pkg_info.get("name").unwrap());
                    let version = get_string(pkg_info.get("version").unwrap());

                    let pkg_name = format!("@{}/{}:{}", ns_pretty, name, version);

                    println!("{} in {}", pkg_name, dir_pretty);
                    if args.long {
                        for (k, v) in pkg_info {
                            if k == "name" || k == "version" {
                                continue;
                            }
                            println!("  {} = {:?}", k, v);
                        }
                    }
                }
            }
        }
    }

    exit(0)
}

fn link_packages(args: LinkPackagesArgs, should_delete: bool) -> ! {
    fn get_string(v: &toml::Value) -> &str {
        match v {
            toml::Value::String(table) => table,
            _ => unreachable!(),
        }
    }

    let world = TypstSystemWorld::new(CompileOpts::default()).unwrap_or_exit();

    let manifest = std::fs::read_to_string(&args.manifest).unwrap();
    let manifest: toml::Table = toml::from_str(&manifest).unwrap();

    let pkg_info = match manifest.get("package").unwrap() {
        toml::Value::Table(table) => table,
        _ => unreachable!(),
    };

    let name = get_string(pkg_info.get("name").unwrap());
    let version = get_string(pkg_info.get("version").unwrap());

    let pkg_dirname = format!("{}/{}", name, version);

    let local_path = world.registry.local_path().unwrap();
    let pkg_link_target = make_absolute(&local_path.join("preview").join(pkg_dirname));
    let pkg_link_source = make_absolute(Path::new(&args.manifest).parent().unwrap());

    let action = if should_delete { "unlink" } else { "link" };

    let src_pretty = unix_slash(&pkg_link_source);
    let dst_pretty = unix_slash(&pkg_link_target);

    eprintln!("{action} package: {} -> {}", src_pretty, dst_pretty);

    if should_delete {
        if !pkg_link_target.exists() {
            eprintln!("package not found");
            exit(1)
        }

        utils::remove_symlink_dir(&pkg_link_target).unwrap();
    } else {
        if pkg_link_target.exists() {
            eprintln!("package already exists");
            exit(1)
        }

        std::fs::create_dir_all(pkg_link_target.parent().unwrap()).unwrap();
        utils::symlink_dir(&pkg_link_source, &pkg_link_target).unwrap();
    }

    exit(0)
}

fn doc_packages(args: GenPackagesDocArgs) -> ! {
    let package_dir = Path::new(&args.manifest).parent().unwrap();
    let doc_file = package_dir.join("doc.typ");

    let compile_args = CompileArgs {
        format: vec!["pdf".to_string(), "svg".to_string()],
        compile: CompileOnceArgs {
            entry: doc_file.to_string_lossy().to_string(),
            workspace: package_dir.to_string_lossy().to_string(),
            output: args.output,
            ..Default::default()
        },
        ..Default::default()
    };

    compile(compile_args)
}
