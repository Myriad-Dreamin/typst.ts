mod tests {
    use sha2::Digest;
    use typst_ts_integration_test::{ArtifactBundle, ArtifactCompiler};

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
                let facts: Facts = bundle_to_facts(&bundle);
                let origin_pdf_hash = &facts.origin_pdf_hash;
                let debug_expr = &format!(
                    "facts.origin_pdf_hash does not match the older one\nOriginalPdfPath: {}",
                    bundle.pdf.display()
                );
                let origin_pdf_hash_fn = $origin_pdf_hash;
                origin_pdf_hash_fn(origin_pdf_hash, debug_expr);
                assert_eq!(
                    facts.origin_pdf_hash, facts.artifact_pdf_hash,
                    "facts.origin_pdf_hash == facts.artifact_pdf_hash"
                );
            };
        }

        check_bundle_facts!(
            compiler.compile("visualize", "visualize/line_1.typ"),
            // origin_pdf_hash
            |origin_pdf_hash: &str, e: &str| insta::assert_snapshot!(origin_pdf_hash, e, @
                r"sha256:f371c03a46ab2823d788525be63673b1a9ac25e526f8b4cdf00a04183da466ca"),
        );

        check_bundle_facts!(
            compiler.compile("visualize", "visualize/line_2.typ"),
            // origin_pdf_hash
            |origin_pdf_hash: &str, e: &str| insta::assert_snapshot!(origin_pdf_hash, e, @
                "sha256:77b4787c8cc10afcf7e23378c13c0ebd0e5829ad884b587695a3d83eb3111c07"),
        );

        // todo: does not preserve outline
        // check_bundle_facts!(
        //     compiler.compile("skyzh-cv", "skyzh-cv/main.typ"),
        //     // origin_pdf_hash
        //     |origin_pdf_hash: &str, e: &str| insta::assert_snapshot!(origin_pdf_hash, e, @
        //         "sha256:b6a2363f54b7cd2fb58660d16b74d1c2931f76c724e87d51edc441a08310a6f1"),
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
