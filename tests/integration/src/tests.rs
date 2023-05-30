mod tests {

    use std::collections::HashMap;

    use anyhow::Ok;
    use base64::Engine;
    use image::codecs::png::PngDecoder;
    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    use typst_ts_integration_test::{wasm::wasm_pack_test, ArtifactBundle, ArtifactCompiler};
    use typst_ts_test_common::package_renderer_dir;

    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestPoint {
        kind: String,
        name: String,
        meta: HashMap<String, String>,
        verbose: HashMap<String, String>,
    }

    fn hash_bytes<T: AsRef<[u8]>>(bytes: T) -> String {
        format!("sha256:{}", hex::encode(sha2::Sha256::digest(bytes)))
    }

    fn hash_image_data_url(data_url: &str) -> String {
        use image_hasher::HasherConfig;

        let data_url = data_url.trim_start_matches("data:image/png;base64,");
        let data = base64::engine::general_purpose::STANDARD
            .decode(data_url)
            .unwrap();

        let image = PngDecoder::new(&data[..]).unwrap();
        let image = image::DynamicImage::from_decoder(image).unwrap();

        let hasher = HasherConfig::new().hash_size(24, 24);
        let hasher = hasher.to_hasher();

        format!(
            "phash-gradient:{}",
            base64::engine::general_purpose::STANDARD.encode(hasher.hash_image(&image).as_bytes())
        )
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

        check_bundle_facts!(
            compiler.compile("visualize", "visualize/path_1.typ"),
            // origin_pdf_hash
            |origin_pdf_hash: &str, e: &str| insta::assert_snapshot!(origin_pdf_hash, e, @
                "sha256:6eae467756cb46021f7d9e826013374e56366186ad14f742a9c8da70ca60d621"),
        );

        check_bundle_facts!(
            compiler.compile("visualize", "visualize/polygon_1.typ"),
            // origin_pdf_hash
            |origin_pdf_hash: &str, e: &str| insta::assert_snapshot!(origin_pdf_hash, e, @
                "sha256:f67821e7b1bf0d170e21e304267846937445070f4bab8a3dbbfec46f67efec73"),
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

    #[test]
    fn test_wasm_renderer_functionality() -> anyhow::Result<()> {
        let artifact_dir = typst_ts_test_common::artifact_dir().join("integrations");

        let res = wasm_pack_test(
            &package_renderer_dir(),
            true,
            &["web_verbose"],
            &["--chrome", "--headless"],
        )?;

        let mut contents = vec![];
        let mut test_points = vec![];

        let mut start_capture = false;
        for line in res.lines() {
            if line.contains(">>> typst_ts_test_capture") {
                start_capture = true;
            } else if line.contains("<<< typst_ts_test_capture") {
                start_capture = false;

                let test_point = serde_json::from_str::<TestPoint>(contents.join("\n").trim())?;
                test_points.push(test_point);
                contents.clear();
            } else if start_capture {
                contents.push(line);
            }
        }

        let mut grouped_test_points = {
            let mut grouped_test_points = HashMap::new();
            for test_point in test_points {
                grouped_test_points
                    .entry(test_point.kind.clone())
                    .or_insert_with(Vec::new)
                    .push(test_point);
            }

            for (_, test_points) in grouped_test_points.iter_mut() {
                test_points.sort_by(|x, y| x.name.cmp(&y.name));
            }

            for canvas_render_test_point in grouped_test_points
                .get_mut("canvas_render_test")
                .ok_or_else(|| anyhow::anyhow!("no test points found"))?
            {
                let data_content = &canvas_render_test_point.verbose["data_content"];
                let data_content_hash = hash_image_data_url(data_content);
                canvas_render_test_point
                    .meta
                    .insert("data_content_phash".to_string(), data_content_hash);
            }
            grouped_test_points
        };

        // store the test points
        let test_points_json = serde_json::to_vec_pretty(&grouped_test_points)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&test_points_json).unwrap();

        let output_dir = artifact_dir.join("renderer");
        let test_points_json_path = output_dir.join("test_points.chrome.json.gz");

        std::fs::create_dir_all(output_dir)?;
        std::fs::write(test_points_json_path, encoder.finish().unwrap())?;

        // check canvas_render_test_points

        let canvas_render_test_points = grouped_test_points.remove("canvas_render_test").unwrap();
        println!(
            "canvas_render_test_points: {:?}",
            canvas_render_test_points.len()
        );
        let mut test_point_iter = canvas_render_test_points.into_iter();

        #[derive(Default, Debug, Serialize, Deserialize)]
        struct Facts {
            name: String,
            data_content_phash: String,
            text_content_hash: String,
        }

        macro_rules! check_canvas_render_test_point {
            (@$snapshot:literal) => {{
                let mut test_point = test_point_iter.next().unwrap();
                let mut filtered_value = Facts::default();

                filtered_value.name = test_point.name.clone();
                filtered_value.data_content_phash = test_point
                    .meta
                    .remove("data_content_phash")
                    .expect("data_content_phash not found");
                filtered_value.text_content_hash = test_point
                    .meta
                    .remove("text_content_hash")
                    .expect("text_content_hash not found");

                let value = insta::_macro_support::serialize_value(
                    &filtered_value,
                    insta::_macro_support::SerializationFormat::Yaml,
                    insta::_macro_support::SnapshotLocation::Inline,
                );
                let data_content = &test_point.verbose["data_content"];
                let text_content = &test_point.verbose["text_content"];
                let debug_expr = &format!(
                    "snapshot does not match the older one\nTestPointName: {}\nDataContent: {}\nTextContent: {}",
                    test_point.name,
                    data_content,
                    text_content
                );
                insta::assert_snapshot!(
                    value,
                    debug_expr,
                    @$snapshot
                );
            }};
        }

        check_canvas_render_test_point!(@r###"
        ---
        name: line_1_artifact_ir
        data_content_phash: "phash-gradient:AQAAAAAACwAACwAACgAALAAADAAAAAAABQAABAAAGQAAZgAAmAAAcQIAxAkAECcAQIwAgDEDAMIEAAgLACALAIAEAAACAAAA"
        text_content_hash: "sha256:ab3d9568e6406923f98df52e373d11781efb1fc4d86eb55fba06d2e1467f8e44"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: line_1_artifact_json
        data_content_phash: "phash-gradient:AQAAAAAACwAACwAACgAALAAADAAAAAAABQAABAAAGQAAZgAAmAAAcQIAxAkAECcAQIwAgDEDAMIEAAgLACALAIAEAAACAAAA"
        text_content_hash: "sha256:ab3d9568e6406923f98df52e373d11781efb1fc4d86eb55fba06d2e1467f8e44"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: line_2_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAQAAJYAAJYAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: line_2_artifact_json
        data_content_phash: "phash-gradient:AAAAAAQAAJYAAJYAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: main_artifact_ir
        data_content_phash: "phash-gradient:AAAAgNwAAMQAmAYA2MYAAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:98c5b7172c1fb068bd716678b1eb9dd73941d9ae5a44fecb2550a970c9407777"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: main_artifact_json
        data_content_phash: "phash-gradient:AAAAgNwAAMQAmAYA2MYAAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:98c5b7172c1fb068bd716678b1eb9dd73941d9ae5a44fecb2550a970c9407777"
        "###);
        // todo: canvas does not paint stroke
        check_canvas_render_test_point!(@r###"
        ---
        name: path_1_artifact_ir
        data_content_phash: "phash-gradient:AAAgAAAACBBgKRBuDRBADTNPjTNGjVVKjTNGTTNNDRBACRNsiVBqDRBAzVBKzVdGSTdFjbNNzZRNDRBAqRBpCBBgAAAAAAAg"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        // todo: canvas does not paint stroke
        check_canvas_render_test_point!(@r###"
        ---
        name: path_1_artifact_json
        data_content_phash: "phash-gradient:AAAgAAAACBBgKRBuDRBADTNPjTNGjVVKjTNGTTNNDRBACRNsiVBqDRBAzVBKzVdGSTdFjbNNzZRNDRBAqRBpCBBgAAAAAAAg"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        // todo: canvas does not paint stroke
        check_canvas_render_test_point!(@r###"
        ---
        name: polygon_1_artifact_ir
        data_content_phash: "phash-gradient:IAAA4BcA4AMAwPE/GOA/wPABwPADAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        // todo: canvas does not paint stroke
        check_canvas_render_test_point!(@r###"
        ---
        name: polygon_1_artifact_json
        data_content_phash: "phash-gradient:IAAA4BcA4AMAwPE/GOA/wPABwPADAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);

        let done = test_point_iter.next();
        if done.is_some() {
            panic!("test_point_iter is not empty: {}", done.unwrap().name);
        }

        Ok(())
    }
}
