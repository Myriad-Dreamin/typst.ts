mod tests {

    use std::collections::HashMap;

    use anyhow::Ok;
    use base64::Engine;
    use image::codecs::png::PngDecoder;
    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    use typst_ts_dev_server::{http::run_http, RunHttpArgs};
    use typst_ts_integration_test::{wasm::wasm_pack_test, ArtifactBundle, ArtifactCompiler};
    use typst_ts_test_common::{corpus_root, package_renderer_dir};

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

        #[derive(Default, Debug, Serialize, Deserialize)]
        struct Facts {
            name: String,
            origin_pdf_hash: String,
            artifact_pdf_hash: String,
        }

        macro_rules! check_bundle_facts {
            ($workspace:expr, $name:expr, @$origin_pdf_hash:literal $(,)?) => {
                let workspace = $workspace.to_string();
                let full_name = format!("{}/{}.typ", workspace, $name);
                let bundle = compiler.compile(workspace, full_name.clone());
                let facts: Facts = bundle_to_facts(full_name, &bundle);
                let value = insta::_macro_support::serialize_value(
                    &facts,
                    insta::_macro_support::SerializationFormat::Yaml,
                    insta::_macro_support::SnapshotLocation::Inline,
                );

                let debug_expr = &format!(
                    "facts does not match the older one\nOriginalPdfPath: {}",
                    bundle.pdf.display()
                );
                insta::assert_snapshot!(
                    value,
                    debug_expr,
                    @$origin_pdf_hash
                );
                assert_eq!(
                    facts.origin_pdf_hash, facts.artifact_pdf_hash,
                    "facts.origin_pdf_hash == facts.artifact_pdf_hash"
                );
            };
        }

        check_bundle_facts!("visualize", "line_1", @r###"
        ---
        name: visualize/line_1.typ
        origin_pdf_hash: "sha256:11e85e7280f5f7e4ac0726a2b181e7806f934654aea732bc65538b8a167b0f3c"
        artifact_pdf_hash: "sha256:11e85e7280f5f7e4ac0726a2b181e7806f934654aea732bc65538b8a167b0f3c"
        "###);
        check_bundle_facts!("visualize", "line_2", @r###"
        ---
        name: visualize/line_2.typ
        origin_pdf_hash: "sha256:2998b95ee4117f0277849eea9e72b2a535d31e16b98703814e3a539a586dedc3"
        artifact_pdf_hash: "sha256:2998b95ee4117f0277849eea9e72b2a535d31e16b98703814e3a539a586dedc3"
        "###);
        check_bundle_facts!("visualize", "path_1", @r###"
        ---
        name: visualize/path_1.typ
        origin_pdf_hash: "sha256:6eae467756cb46021f7d9e826013374e56366186ad14f742a9c8da70ca60d621"
        artifact_pdf_hash: "sha256:6eae467756cb46021f7d9e826013374e56366186ad14f742a9c8da70ca60d621"
        "###);
        check_bundle_facts!("visualize", "polygon_1", @r###"
        ---
        name: visualize/polygon_1.typ
        origin_pdf_hash: "sha256:cf52d8b5714a727217ef159423acc1c6f8848c1ef9f95ffbd3135a242e420799"
        artifact_pdf_hash: "sha256:cf52d8b5714a727217ef159423acc1c6f8848c1ef9f95ffbd3135a242e420799"
        "###);

        // todo: does not preserve outline
        // check_bundle_facts!("skyzh-cv", "main", @"sha256:b6a2363f54b7cd2fb58660d16b74d1c2931f76c724e87d51edc441a08310a6f1");

        check_bundle_facts!("visualize", "shape_aspect_1", @r###"
        ---
        name: visualize/shape_aspect_1.typ
        origin_pdf_hash: "sha256:a0289f41ec2f4202d2493eae027b5cc98b991d56b69be995f38e3e517aa4480e"
        artifact_pdf_hash: "sha256:a0289f41ec2f4202d2493eae027b5cc98b991d56b69be995f38e3e517aa4480e"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_2", @r###"
        ---
        name: visualize/shape_aspect_2.typ
        origin_pdf_hash: "sha256:3bc8a7216a5b8ef851dcf8766a750c581e2fffee6eba5e58eab947f9e72adf9d"
        artifact_pdf_hash: "sha256:3bc8a7216a5b8ef851dcf8766a750c581e2fffee6eba5e58eab947f9e72adf9d"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_3", @r###"
        ---
        name: visualize/shape_aspect_3.typ
        origin_pdf_hash: "sha256:4bddcc9236aaff4cebf1f36ad16baa8abe4cbb36ba01d6cc4428105a6acaae51"
        artifact_pdf_hash: "sha256:4bddcc9236aaff4cebf1f36ad16baa8abe4cbb36ba01d6cc4428105a6acaae51"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_4", @r###"
        ---
        name: visualize/shape_aspect_4.typ
        origin_pdf_hash: "sha256:65472e6c14c510305714e39912be43a23c936ac550ad750c36e9e223c77efc2f"
        artifact_pdf_hash: "sha256:65472e6c14c510305714e39912be43a23c936ac550ad750c36e9e223c77efc2f"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_5", @r###"
        ---
        name: visualize/shape_aspect_5.typ
        origin_pdf_hash: "sha256:6e914b8908e90a69d37a16ffcd27132efe986eeca6bf3c196e671bfe04ebbb3a"
        artifact_pdf_hash: "sha256:6e914b8908e90a69d37a16ffcd27132efe986eeca6bf3c196e671bfe04ebbb3a"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_6", @r###"
        ---
        name: visualize/shape_aspect_6.typ
        origin_pdf_hash: "sha256:4abd19dbcb71a051d02bb929fdf71c5372960a2c64c4bf2ddb711618fca57658"
        artifact_pdf_hash: "sha256:4abd19dbcb71a051d02bb929fdf71c5372960a2c64c4bf2ddb711618fca57658"
        "###);

        check_bundle_facts!("text", "chinese", @r###"
        ---
        name: text/chinese.typ
        origin_pdf_hash: "sha256:2da42395330a6bd2114c9b70dc21447b32613fdbcb06f523e885603331bdf3ad"
        artifact_pdf_hash: "sha256:2da42395330a6bd2114c9b70dc21447b32613fdbcb06f523e885603331bdf3ad"
        "###);

        check_bundle_facts!("text", "deco_1", @r###"
        ---
        name: text/deco_1.typ
        origin_pdf_hash: "sha256:5759095c93bb58ecb31468cc3cfb32e0e02a65a82331415d7f93fe76ea40be1f"
        artifact_pdf_hash: "sha256:5759095c93bb58ecb31468cc3cfb32e0e02a65a82331415d7f93fe76ea40be1f"
        "###);
        // todo: figure out why rgba does not work
        check_bundle_facts!("text", "deco_2", @r###"
        ---
        name: text/deco_2.typ
        origin_pdf_hash: "sha256:75a06b1b425917b6486b27eaba2db1bb42efc1ae280dd80f8fc766d34db0209b"
        artifact_pdf_hash: "sha256:75a06b1b425917b6486b27eaba2db1bb42efc1ae280dd80f8fc766d34db0209b"
        "###);
        check_bundle_facts!("text", "deco_3", @r###"
        ---
        name: text/deco_3.typ
        origin_pdf_hash: "sha256:7c67e578dd76f0571558c127253ecb64586d93481198bf094fff900974e90a92"
        artifact_pdf_hash: "sha256:7c67e578dd76f0571558c127253ecb64586d93481198bf094fff900974e90a92"
        "###);

        check_bundle_facts!("text", "emoji_1", @r###"
        ---
        name: text/emoji_1.typ
        origin_pdf_hash: "sha256:a22cc8f238cc29b43c0b705fe65fd72ed0eb2a457f67e1f0cd279e562b29212f"
        artifact_pdf_hash: "sha256:a22cc8f238cc29b43c0b705fe65fd72ed0eb2a457f67e1f0cd279e562b29212f"
        "###);
        check_bundle_facts!("text", "emoji_2", @r###"
        ---
        name: text/emoji_2.typ
        origin_pdf_hash: "sha256:9c5e3b9668a1dfa72a5206be82074edb2ed89ebfb3c18bfdbc5d0a8360968925"
        artifact_pdf_hash: "sha256:9c5e3b9668a1dfa72a5206be82074edb2ed89ebfb3c18bfdbc5d0a8360968925"
        "###);

        fn bundle_to_facts(name: String, bundle: &ArtifactBundle) -> Facts {
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
                name,
                artifact_pdf_hash,
                origin_pdf_hash,
            }
        }
    }

    #[tokio::test]
    async fn test_wasm_renderer_functionality() -> anyhow::Result<()> {
        tokio::spawn(run_http(RunHttpArgs {
            corpus: corpus_root(),
            http: "127.0.0.1:20810".to_owned(),
        }));
        tokio::spawn(test_wasm_renderer_functionality_main())
            .await
            .unwrap()
    }

    async fn test_wasm_renderer_functionality_main() -> anyhow::Result<()> {
        let artifact_dir = typst_ts_test_common::artifact_dir().join("integrations");

        let res = wasm_pack_test(
            &package_renderer_dir(),
            true,
            &["web_verbose"],
            &["--chrome", "--headless"],
        )
        .await?;

        let mut contents = vec![];
        let mut rest_contents = vec![];
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
            } else {
                rest_contents.push(line);
            }
        }

        println!("::group::Output of wasm-pack test");
        println!("{}", rest_contents.join("\n"));
        println!("::endgroup::");

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
                    "\n::group::Snapshot testing Failure (Browser Canvas Rendering)\nsnapshot does not match the older one\nTestPointName: {}\nDataContent: {}\nTextContent: {}\n::endgroup::",
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
        name: math_main_artifact_ir
        data_content_phash: "phash-gradient:AAAAgMwAAMQAmAYA2MYAAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:98c5b7172c1fb068bd716678b1eb9dd73941d9ae5a44fecb2550a970c9407777"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: math_main_artifact_json
        data_content_phash: "phash-gradient:AAAAgMwAAMQAmAYA2MYAAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:98c5b7172c1fb068bd716678b1eb9dd73941d9ae5a44fecb2550a970c9407777"
        "###);
        // todo: the size of cjk font file is quite big
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_chinese_artifact_ir
        // data_content_phash: "phash-gradient:KKprrKlq6Kxm0KTmZKpaZIrbNGI0pNI0tZI1qBI1rDy1bIpqLJpjqFU2qFUlVFS1hIkalIkasGoasKpStWhmpGhmiCoGqZYE"
        // text_content_hash: "sha256:08633df6b8b06027fee154dccd3d530fd53db36851c597621c8e8b65e52b028b"
        // "###);
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_chinese_artifact_json
        // data_content_phash: "phash-gradient:KKprrKlq6Kxm0KTmZKpaZIrbNGI0pNI0tZI1qBI1rDy1bIpqLJpjqFU2qFUlVFS1hIkalIkasGoasKpStWhmpGhmiCoGqZYE"
        // text_content_hash: "sha256:08633df6b8b06027fee154dccd3d530fd53db36851c597621c8e8b65e52b028b"
        // "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_1_artifact_ir
        data_content_phash: "phash-gradient:AQAAVMntXEvqoSACLGlFbHkAAgAAdFsAaBsChAAAUMoGUKEWKAoAmFwEQBsADBQESLQWQCQGJIkDxHYLkHYDdAAAxAICAAAA"
        text_content_hash: "sha256:b6cd67635583e3986fe23c75e130a51494f1682b390d08186efcf949a4d34a23"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_1_artifact_json
        data_content_phash: "phash-gradient:AQAAVMntXEvqoSACLGlFbHkAAgAAdFsAaBsChAAAUMoGUKEWKAoAmFwEQBsADBQESLQWQCQGJIkDxHYLkHYDdAAAxAICAAAA"
        text_content_hash: "sha256:b6cd67635583e3986fe23c75e130a51494f1682b390d08186efcf949a4d34a23"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_2_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAEAAAAEBIMRAAs4IssNMsskJosAQAscFIRpEEJYNUWZPcWYOcGZGAoAZAS5AZtRBZtRBxzxBh5pBlgAQQRAAAAAAAA"
        text_content_hash: "sha256:ad356092282c0389bf6aebee517125c4c1faf68edb4e0ef0a9b8b41333c11454"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_2_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAEAAAAEBIMRAAs4IssNMsskJosAQAscFIRpEEJYNUWZPcWYOcGZGAoAZAS5AZtRBZtRBxzxBh5pBlgAQQRAAAAAAAA"
        text_content_hash: "sha256:ad356092282c0389bf6aebee517125c4c1faf68edb4e0ef0a9b8b41333c11454"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_3_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAABAEAAAAAhAMAhAsAMAsAtAMAKAMAZAEAlAsAlAMAZAsAVAsAAAAAAAEAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:39d1a4c5927c6248493727c5b08ffc36f22665ec3dc7581e247232970ca4edad"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_3_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAABAEAAAAAhAMAhAsAMAsAtAMAKAMAZAEAlAsAlAMAZAsAVAsAAAAAAAEAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:39d1a4c5927c6248493727c5b08ffc36f22665ec3dc7581e247232970ca4edad"
        "###);
        // still inconsisistent
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_emoji_1_artifact_ir
        // data_content_phash: "phash-gradient:AABAwKdFQLZFyIdFwINFAABCAABAQIBBSKBFSKBFQKBBAABAgMFBAAZAAPZBANFFANJCAABBAIBBAABAQCJLCCdLCCZLANhD"
        // text_content_hash: "sha256:e96d18327a60513e2375c2dfa12d17872c97304df451630781965a6ae8031b45"
        // "###);
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_emoji_1_artifact_json
        // data_content_phash: "phash-gradient:AABAwKdFQLZFyIdFwINFAABCAABAQIBBSKBFSKBFQKBBAABAgMFBAAZAAPZBANFFANJCAABBAIBBAABAQCJLCCdLCCZLANhD"
        // text_content_hash: "sha256:e96d18327a60513e2375c2dfa12d17872c97304df451630781965a6ae8031b45"
        // "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_emoji_2_artifact_ir
        data_content_phash: "phash-gradient:AABAAABAAABAAABAAAhBAABAkJlzAE4DadppcZhxlZhVkphRlJhTFJhTFMhTlItTkosykos2hKNGkKFWgKRWYABIAAAAAABA"
        text_content_hash: "sha256:8e46fa236bdfb74c3259b7a186191a5febfafd5c3e38f5b2e110931114486ade"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_emoji_2_artifact_json
        data_content_phash: "phash-gradient:AABAAABAAABAAABAAAhBAABAkJlzAE4DadppcZhxlZhVkphRlJhTFJhTFMhTlItTkosykos2hKNGkKFWgKRWYABIAAAAAABA"
        text_content_hash: "sha256:8e46fa236bdfb74c3259b7a186191a5febfafd5c3e38f5b2e110931114486ade"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_1_artifact_ir
        data_content_phash: "phash-gradient:JAEAAAIAJAEEJIcURAYEQRMXskAHBAAABAACCQAANgAATAAAMQEA4gYAiAkAICYAQMwAADEBAMQCAIgFACABAEAAAAABAAAA"
        text_content_hash: "sha256:ab3d9568e6406923f98df52e373d11781efb1fc4d86eb55fba06d2e1467f8e44"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_1_artifact_json
        data_content_phash: "phash-gradient:JAEAAAIAJAEEJIcURAYEQRMXskAHBAAABAACCQAANgAATAAAMQEA4gYAiAkAICYAQMwAADEBAMQCAIgFACABAEAAAAABAAAA"
        text_content_hash: "sha256:ab3d9568e6406923f98df52e373d11781efb1fc4d86eb55fba06d2e1467f8e44"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_2_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAgAAJIBgE0CgE0CgE0CgE0CgE0CgE0CgE0CgE0AABIBAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_2_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAgAAJIBgE0CgE0CgE0CgE0CgE0CgE0CgE0CgE0AABIBAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_path_1_artifact_ir
        data_content_phash: "phash-gradient:AAAgAAAACBBgKRBuDRBADTNNjTNGjVVCjTNGTTNNDRBACTNtqVBqDRBAzVFKyVdGSTZFybNNzZVNTRBAKRFpSBBgAAAAAAAg"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_path_1_artifact_json
        data_content_phash: "phash-gradient:AAAgAAAACBBgKRBuDRBADTNNjTNGjVVCjTNGTTNNDRBACTNtqVBqDRBAzVFKyVdGSTZFybNNzZVNTRBAKRFpSBBgAAAAAAAg"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_polygon_1_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAA4AAAmAEAYAAA4AQAiAEAkBcA4AcAYAAA4AIAYAAA0EEomAEAAPx/BPB/MAAAOTAiMEQAALADcDAAwPYHwLADOTAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_polygon_1_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAA4AAAmAEAYAAA4AQAiAEAkBcA4AcAYAAA4AIAYAAA0EEomAEAAPx/BPB/MAAAOTAiMEQAALADcDAAwPYHwLADOTAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_1_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAgABFoABjAABhAABhAABhAABhAwBhBCBhCwlhKpljQtljXsBjTZhjSNBjStBjSlFjSlBjCMFjSNBEjSAQAAAAjC"
        text_content_hash: "sha256:5da5da474534aac328d56672de9236f5246664aeb03244b3432fbefe9daf2878"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_1_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAgABFoABjAABhAABhAABhAABhAwBhBCBhCwlhKpljQtljXsBjTZhjSNBjStBjSlFjSlBjCMFjSNBEjSAQAAAAjC"
        text_content_hash: "sha256:5da5da474534aac328d56672de9236f5246664aeb03244b3432fbefe9daf2878"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_2_artifact_ir
        data_content_phash: "phash-gradient:AABAADBAAkBACbRATTZJFKEgNC1MZKdRRIAgHEVpDGRJDGRADCRODCROjGVMDGZAjGNrjMMkRKAwrTRZSTQsAoBQAAAAAABA"
        text_content_hash: "sha256:2ab9062c19279ff04df938b822b023cbef3f0d8c09d7b956781f58aff9ee86af"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_2_artifact_json
        data_content_phash: "phash-gradient:AABAADBAAkBACbRATTZJFKEgNC1MZKdRRIAgHEVpDGRJDGRADCRODCROjGVMDGZAjGNrjMMkRKAwrTRZSTQsAoBQAAAAAABA"
        text_content_hash: "sha256:2ab9062c19279ff04df938b822b023cbef3f0d8c09d7b956781f58aff9ee86af"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_3_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAYAMAgAQAaAsAJBcAhhQAxBQAxAwAxAwAxC0AxC0AxAwAxAwAxBQAhhQAJAcAaAsAgAQAYAMAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_3_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAYAMAgAQAaAsAJBcAhhQAxBQAxAwAxAwAxC0AxC0AxAwAxAwAxBQAhhQAJAcAaAsAgAQAYAMAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_4_artifact_ir
        data_content_phash: "phash-gradient:AFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_4_artifact_json
        data_content_phash: "phash-gradient:AFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_5_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAKAEAEgIAKQUAKQsA1AoA1AoA1AoA1AoAKQsAKQUAEgIAKAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_5_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAKAEAEgIAKQUAKQsA1AoA1AoA1AoA1AoAKQsAKQUAEgIAKAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        // todo: double page
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_6_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAABAAAAAAAVoAKTIACTAAKTIAKTAAKTAAKTAAKTAAKTAAKTAAKTAAKTAAKTIACTAAKTIAAVoAAAAAABAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_6_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAABAAAAAAAVoAKTIACTAAKTIAKTAAKTAAKTAAKTAAKTAAKTAAKTAAKTAAKTIACTAAKTIAAVoAAAAAABAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);

        let done = test_point_iter.next();
        if done.is_some() {
            panic!("test_point_iter is not empty: {}", done.unwrap().name);
        }

        Ok(())
    }
}
