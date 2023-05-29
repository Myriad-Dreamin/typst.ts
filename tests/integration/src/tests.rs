mod tests {
    use sha2::Digest;
    use std::path::Path;

    use typst::{doc::Document, util::PathExt};
    use typst_ts_compiler::{service::CompileDriver, TypstSystemWorld};
    use typst_ts_core::{
        config::CompileOpts,
        exporter_builtins::{FromExporter, FsPathExporter, GroupExporter},
        AsWritable,
    };
    use typst_ts_pdf_exporter::PdfDocExporter;
    use typst_ts_serde_exporter::JsonArtifactExporter;
    use typst_ts_tir_exporter::IRArtifactExporter;

    fn get_driver(
        workspace_dir: &Path,
        entry_file_path: &Path,
        exporter: GroupExporter<Document>,
    ) -> CompileDriver {
        let world = TypstSystemWorld::new(CompileOpts {
            root_dir: workspace_dir.to_owned(),
            no_system_fonts: true,
            ..CompileOpts::default()
        });

        CompileDriver {
            world,
            entry_file: entry_file_path.to_owned(),
            exporter,
        }
    }

    macro_rules! artifact_exporters {
        ($($exporters:expr),*) => {
            {
                let artifact_exporters: Vec<Box<dyn typst_ts_core::Exporter<typst_ts_core::Artifact>>> = vec![
                    $(Box::new($exporters)),*
                ];
                FromExporter::new(artifact_exporters)
            }
        };
    }

    macro_rules! document_exporters {
        ($($exporters:expr),*) => {
            {
                let document_exporters: Vec<Box<dyn typst_ts_core::Exporter<typst::doc::Document>>> = vec![
                    $(Box::new($exporters)),*
                ];
                GroupExporter::new(document_exporters)
            }
        };
    }

    macro_rules! ir_exporters {
        ($($exporters:expr),*) => {
            {
                let ir_exporters: Vec<Box<dyn typst_ts_core::Exporter<typst_ts_core::artifact_ir::Artifact>>> = vec![
                    $(Box::new($exporters)),*
                ];
                FromExporter::new(ir_exporters)
            }
        };
    }

    pub fn artifact_json_to_path<P: AsRef<Path>>(
        path: P,
    ) -> FsPathExporter<AsWritable, JsonArtifactExporter> {
        FsPathExporter::new(path.as_ref().to_owned(), JsonArtifactExporter::default())
    }

    pub fn artifact_ir_to_path<P: AsRef<Path>>(
        path: P,
    ) -> FsPathExporter<AsWritable, IRArtifactExporter> {
        FsPathExporter::new(path.as_ref().to_owned(), IRArtifactExporter::default())
    }

    pub fn doc_pdf_to_path<P: AsRef<Path>>(path: P) -> FsPathExporter<Vec<u8>, PdfDocExporter> {
        FsPathExporter::new(path.as_ref().to_owned(), PdfDocExporter::default())
    }

    pub struct ArtifactBundle {
        pub driver: CompileDriver,
        pub json: std::path::PathBuf,
        pub tir: std::path::PathBuf,
        pub pdf: std::path::PathBuf,
    }

    pub struct ArtifactCompiler {
        pub corpus_root: std::path::PathBuf,
        pub artifact_dir: std::path::PathBuf,
    }

    impl ArtifactCompiler {
        pub fn compile(
            &self,
            workspace_dir: &'static str,
            entry_file: &'static str,
        ) -> ArtifactBundle {
            let entry_file_base = Path::new(entry_file);

            let real_entry_file_path = self.corpus_root.join(entry_file_base);
            let real_workspace_dir = self.corpus_root.join(workspace_dir);

            let artifact_dir = &self.artifact_dir;

            let json_artifact_file_path =
                artifact_dir.join(entry_file_base.with_extension("artifact.json"));
            let tir_file_path =
                artifact_dir.join(entry_file_base.with_extension("artifact.tir.bin"));
            let pdf_file_path = artifact_dir.join(entry_file_base.with_extension("pdf"));

            let artifact_dir_to_create = json_artifact_file_path.parent().unwrap().to_owned();
            std::fs::create_dir_all(artifact_dir_to_create).unwrap();

            let mut driver = get_driver(
                &real_workspace_dir,
                &real_entry_file_path,
                document_exporters![
                    artifact_exporters![artifact_json_to_path(json_artifact_file_path.clone())],
                    ir_exporters![artifact_ir_to_path(tir_file_path.clone())],
                    doc_pdf_to_path(pdf_file_path.clone())
                ],
            );

            driver.once().unwrap();

            ArtifactBundle {
                driver,
                json: json_artifact_file_path.normalize(),
                tir: tir_file_path.normalize(),
                pdf: pdf_file_path.normalize(),
            }
        }
    }

    fn hash_bytes<T: AsRef<[u8]>>(bytes: T) -> String {
        format!("sha256:{}", hex::encode(sha2::Sha256::digest(bytes)))
    }

    #[test]
    fn test_local_consistency() {
        let corpus_root = typst_ts_test_common::corpus_root();
        let artifact_dir = typst_ts_test_common::artifact_dir().join("integrations");

        let compiler = ArtifactCompiler {
            corpus_root,
            artifact_dir,
        };

        struct Facts {
            origin_pdf_hash: String,
            artifact_pdf_hash: String,
        }

        macro_rules! check_bundle_facts {
            ($bundle:expr, $origin_pdf_hash:expr, ) => {
                let bundle = $bundle;
                let origin_pdf_hash = $origin_pdf_hash;
                let facts: Facts = bundle_to_facts(&bundle);
                let debug_expr = format!("facts.origin_pdf_hash does not match the older one\nOriginalPdfPath: {}", bundle.pdf.display());
                origin_pdf_hash(&facts.origin_pdf_hash, &debug_expr);
                assert_eq!(facts.origin_pdf_hash, facts.artifact_pdf_hash, "facts.origin_pdf_hash == facts.artifact_pdf_hash");
            }
        }

        check_bundle_facts!(
            compiler.compile("visualize", "visualize/line_1.typ"),
            // origin_pdf_hash
            |origin_pdf_hash: &str, debug_expr: &str| insta::assert_snapshot!(origin_pdf_hash, debug_expr, 
                @r"sha256:f371c03a46ab2823d788525be63673b1a9ac25e526f8b4cdf00a04183da466ca"),
        );

        check_bundle_facts!(
            compiler.compile("visualize", "visualize/line_2.typ"),
            // origin_pdf_hash
            |origin_pdf_hash: &str, debug_expr: &str| insta::assert_snapshot!(origin_pdf_hash, debug_expr, 
                @"sha256:77b4787c8cc10afcf7e23378c13c0ebd0e5829ad884b587695a3d83eb3111c07"),
        );

        // todo: does not preserve outline
        // check_bundle_facts!(
        //     compiler.compile("skyzh-cv", "skyzh-cv/main.typ"),
        //     // origin_pdf_hash
        //     |origin_pdf_hash: &str, debug_expr: &str| insta::assert_snapshot!(origin_pdf_hash, debug_expr, 
        //         @"sha256:b6a2363f54b7cd2fb58660d16b74d1c2931f76c724e87d51edc441a08310a6f1"),
        // );

        fn bundle_to_facts(bundle: &ArtifactBundle) -> Facts {
            let json_artifact = std::fs::read(&bundle.json).unwrap();
            let json_artifact = serde_json::from_slice::<typst_ts_core::Artifact>(&json_artifact)
                .expect("failed to deserialize json artifact");

            let doc = json_artifact.to_document(&bundle.driver.world.font_resolver);
            let pdf_doc = typst::export::pdf(&doc);

            let pdf_path = bundle.pdf.with_extension("artifact.pdf");
            std::fs::write(pdf_path, &pdf_doc).unwrap();

            let origin_doc = std::fs::read(&bundle.pdf).unwrap();

            let artifact_pdf_hash = hash_bytes(pdf_doc);
            let origin_pdf_hash = hash_bytes(origin_doc);

            Facts {
                artifact_pdf_hash,
                origin_pdf_hash,
            }
        }
    }
}
