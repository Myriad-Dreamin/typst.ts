mod tests {

    use std::collections::HashMap;

    use anyhow::Ok;
    use base64::Engine;
    use image::codecs::png::PngDecoder;
    use serde::{Deserialize, Serialize};
    use typst_ts_dev_server::{http::run_http, RunHttpArgs};
    use typst_ts_integration_test::wasm::wasm_pack_test;
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

    // not valid anymore
    #[test]
    #[cfg(feature = "disabled")]
    fn test_local_consistency() {
        fn hash_bytes<T: AsRef<[u8]>>(bytes: T) -> String {
            format!("sha256:{}", hex::encode(sha2::Sha256::digest(bytes)))
        }

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

        check_bundle_facts!("layout", "clip_1", @r###"
        ---
        name: layout/clip_1.typ
        origin_pdf_hash: "sha256:cfeee7c32bef59ee25284442ef46855430c289295007f03edcf1ace2f5e12e5c"
        artifact_pdf_hash: "sha256:cfeee7c32bef59ee25284442ef46855430c289295007f03edcf1ace2f5e12e5c"
        "###);
        check_bundle_facts!("layout", "clip_2", @r###"
        ---
        name: layout/clip_2.typ
        origin_pdf_hash: "sha256:ebe45489f2f471094112c427c9aee520de24d0333af5a18b951dcccc5e1c48d9"
        artifact_pdf_hash: "sha256:ebe45489f2f471094112c427c9aee520de24d0333af5a18b951dcccc5e1c48d9"
        "###);
        check_bundle_facts!("layout", "clip_3", @r###"
        ---
        name: layout/clip_3.typ
        origin_pdf_hash: "sha256:e15bc75b69ab8e37c38f7c4882b6bfe4156ce21fbf005df212833669e8749828"
        artifact_pdf_hash: "sha256:e15bc75b69ab8e37c38f7c4882b6bfe4156ce21fbf005df212833669e8749828"
        "###);
        check_bundle_facts!("layout", "clip_4", @r###"
        ---
        name: layout/clip_4.typ
        origin_pdf_hash: "sha256:5b6ea45846ece4de47a73a350abdf8f8f147be13df3d29d2de929fb0f6dc5629"
        artifact_pdf_hash: "sha256:5b6ea45846ece4de47a73a350abdf8f8f147be13df3d29d2de929fb0f6dc5629"
        "###);

        check_bundle_facts!("layout", "list_marker_1", @r###"
        ---
        name: layout/list_marker_1.typ
        origin_pdf_hash: "sha256:4164254fc4f28c66f1338d506a42b680af24b5286c491058a0d474562e31cb11"
        artifact_pdf_hash: "sha256:4164254fc4f28c66f1338d506a42b680af24b5286c491058a0d474562e31cb11"
        "###);
        check_bundle_facts!("layout", "list_marker_2", @r###"
        ---
        name: layout/list_marker_2.typ
        origin_pdf_hash: "sha256:4f056160819ebd0e51388b249def3bf5e8c12519370c0a3c5faa7fa02ed84684"
        artifact_pdf_hash: "sha256:4f056160819ebd0e51388b249def3bf5e8c12519370c0a3c5faa7fa02ed84684"
        "###);
        check_bundle_facts!("layout", "list_marker_3", @r###"
        ---
        name: layout/list_marker_3.typ
        origin_pdf_hash: "sha256:765f999bda1a2bbabc3a9c59485ee394906dd7addb125939a9a13542a4621fec"
        artifact_pdf_hash: "sha256:765f999bda1a2bbabc3a9c59485ee394906dd7addb125939a9a13542a4621fec"
        "###);
        check_bundle_facts!("layout", "list_marker_4", @r###"
        ---
        name: layout/list_marker_4.typ
        origin_pdf_hash: "sha256:e95e86d140bfb2f067ee684ee6701cd6ba26138580b8bada019010667dcd5048"
        artifact_pdf_hash: "sha256:e95e86d140bfb2f067ee684ee6701cd6ba26138580b8bada019010667dcd5048"
        "###);

        check_bundle_facts!("layout", "transform_1", @r###"
        ---
        name: layout/transform_1.typ
        origin_pdf_hash: "sha256:a455739a49b965b400aee08e9f3402cfebb44b8d3c5b40037c1d11e7bc6ddfea"
        artifact_pdf_hash: "sha256:a455739a49b965b400aee08e9f3402cfebb44b8d3c5b40037c1d11e7bc6ddfea"
        "###);
        check_bundle_facts!("layout", "transform_2", @r###"
        ---
        name: layout/transform_2.typ
        origin_pdf_hash: "sha256:40f02a3a903fb30cc48beeb608590c97a04e6fabf8a4e37f9719d3a82e5118ae"
        artifact_pdf_hash: "sha256:40f02a3a903fb30cc48beeb608590c97a04e6fabf8a4e37f9719d3a82e5118ae"
        "###);
        check_bundle_facts!("layout", "transform_3", @r###"
        ---
        name: layout/transform_3.typ
        origin_pdf_hash: "sha256:3c6a87f0002d995952b661188f8320a9d1917dcbcfbcce808dce6a6b32f74991"
        artifact_pdf_hash: "sha256:3c6a87f0002d995952b661188f8320a9d1917dcbcfbcce808dce6a6b32f74991"
        "###);
        check_bundle_facts!("layout", "transform_4", @r###"
        ---
        name: layout/transform_4.typ
        origin_pdf_hash: "sha256:6507a6bc34f0a3f507261953bcadbfa9ffd4e12bec0d2334b6e2997510af2de7"
        artifact_pdf_hash: "sha256:6507a6bc34f0a3f507261953bcadbfa9ffd4e12bec0d2334b6e2997510af2de7"
        "###);

        check_bundle_facts!("visualize", "line_1", @r###"
        ---
        name: visualize/line_1.typ
        origin_pdf_hash: "sha256:441ac5c31daa5345f0106582f3373ffc254fc62ea5f5bcd7f9954e2169a80338"
        artifact_pdf_hash: "sha256:441ac5c31daa5345f0106582f3373ffc254fc62ea5f5bcd7f9954e2169a80338"
        "###);
        check_bundle_facts!("visualize", "line_2", @r###"
        ---
        name: visualize/line_2.typ
        origin_pdf_hash: "sha256:2ad4012029fbf490f7500fdc0eb2288850defa474b6d35bcbc8428c2fa4fa316"
        artifact_pdf_hash: "sha256:2ad4012029fbf490f7500fdc0eb2288850defa474b6d35bcbc8428c2fa4fa316"
        "###);
        check_bundle_facts!("visualize", "path_1", @r###"
        ---
        name: visualize/path_1.typ
        origin_pdf_hash: "sha256:bdd63662ddf4b45cd9408a09da491a87168864bb558c6125839eefc62d43d5d4"
        artifact_pdf_hash: "sha256:bdd63662ddf4b45cd9408a09da491a87168864bb558c6125839eefc62d43d5d4"
        "###);
        check_bundle_facts!("visualize", "polygon_1", @r###"
        ---
        name: visualize/polygon_1.typ
        origin_pdf_hash: "sha256:1b0b1ccb67a2889627c4adb6ae27396de700b9fb476c567a3117e15c2d311a1c"
        artifact_pdf_hash: "sha256:1b0b1ccb67a2889627c4adb6ae27396de700b9fb476c567a3117e15c2d311a1c"
        "###);

        // todo: does not preserve outline
        // check_bundle_facts!("skyzh-cv", "main",
        // @"sha256:b6a2363f54b7cd2fb58660d16b74d1c2931f76c724e87d51edc441a08310a6f1");

        check_bundle_facts!("visualize", "shape_aspect_1", @r###"
        ---
        name: visualize/shape_aspect_1.typ
        origin_pdf_hash: "sha256:e66f4aa150a59fafbb23552e50953e805574ccbdde6341151d67b655e4215894"
        artifact_pdf_hash: "sha256:e66f4aa150a59fafbb23552e50953e805574ccbdde6341151d67b655e4215894"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_2", @r###"
        ---
        name: visualize/shape_aspect_2.typ
        origin_pdf_hash: "sha256:55668a27965507a5ecc3d5d76670e99f0229e4306959ed832ed14037648cd261"
        artifact_pdf_hash: "sha256:55668a27965507a5ecc3d5d76670e99f0229e4306959ed832ed14037648cd261"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_3", @r###"
        ---
        name: visualize/shape_aspect_3.typ
        origin_pdf_hash: "sha256:bfdd05bb4e504472fe1f16272d189a7926665ffe31ba8edb73fbc0012ac629bd"
        artifact_pdf_hash: "sha256:bfdd05bb4e504472fe1f16272d189a7926665ffe31ba8edb73fbc0012ac629bd"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_4", @r###"
        ---
        name: visualize/shape_aspect_4.typ
        origin_pdf_hash: "sha256:e03847b6ea9202bff0a3b3bad8a4d6b773a131e4570569f5dadaf2da0f252590"
        artifact_pdf_hash: "sha256:e03847b6ea9202bff0a3b3bad8a4d6b773a131e4570569f5dadaf2da0f252590"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_5", @r###"
        ---
        name: visualize/shape_aspect_5.typ
        origin_pdf_hash: "sha256:54f7fee4397628d8e64b829471788211d2f7b24848ea3fadbcaffaf45dcfd9a9"
        artifact_pdf_hash: "sha256:54f7fee4397628d8e64b829471788211d2f7b24848ea3fadbcaffaf45dcfd9a9"
        "###);
        check_bundle_facts!("visualize", "shape_aspect_6", @r###"
        ---
        name: visualize/shape_aspect_6.typ
        origin_pdf_hash: "sha256:64eb2b3ee34f99174e264137d172e605c5a57f1deeb3bf8e8ecfd450596355b6"
        artifact_pdf_hash: "sha256:64eb2b3ee34f99174e264137d172e605c5a57f1deeb3bf8e8ecfd450596355b6"
        "###);
        check_bundle_facts!("visualize", "shape_circle_1", @r###"
        ---
        name: visualize/shape_circle_1.typ
        origin_pdf_hash: "sha256:5ba8d5b24e22993ba9bf69bd3a527e70305c3e1676c1c5955411647d1ada8cd3"
        artifact_pdf_hash: "sha256:5ba8d5b24e22993ba9bf69bd3a527e70305c3e1676c1c5955411647d1ada8cd3"
        "###);
        check_bundle_facts!("visualize", "shape_circle_2", @r###"
        ---
        name: visualize/shape_circle_2.typ
        origin_pdf_hash: "sha256:d9f7900e14d38cf7d7b6b96ba9f0cb1b0a96e8572362ccd8b1265ad14dc1e84c"
        artifact_pdf_hash: "sha256:d9f7900e14d38cf7d7b6b96ba9f0cb1b0a96e8572362ccd8b1265ad14dc1e84c"
        "###);
        check_bundle_facts!("visualize", "shape_circle_3", @r###"
        ---
        name: visualize/shape_circle_3.typ
        origin_pdf_hash: "sha256:edfe4cdc7338ab8c124fd8c76d623efa9fc0d94342a2bb932e310369bc7f505e"
        artifact_pdf_hash: "sha256:edfe4cdc7338ab8c124fd8c76d623efa9fc0d94342a2bb932e310369bc7f505e"
        "###);
        check_bundle_facts!("visualize", "shape_circle_4", @r###"
        ---
        name: visualize/shape_circle_4.typ
        origin_pdf_hash: "sha256:7656b2956c6a438045e144860420461d63297263a596060fa4365cb5a0670565"
        artifact_pdf_hash: "sha256:7656b2956c6a438045e144860420461d63297263a596060fa4365cb5a0670565"
        "###);
        // todo: typst cannot pass visualize/stroke_4 test.
        check_bundle_facts!("visualize", "stroke_1", @r###"
        ---
        name: visualize/stroke_1.typ
        origin_pdf_hash: "sha256:520eb4e544f583f68ded37ea6e348bfdd4abcd3746761b1a6c709ff5d5d8cd98"
        artifact_pdf_hash: "sha256:520eb4e544f583f68ded37ea6e348bfdd4abcd3746761b1a6c709ff5d5d8cd98"
        "###);
        check_bundle_facts!("visualize", "stroke_2", @r###"
        ---
        name: visualize/stroke_2.typ
        origin_pdf_hash: "sha256:7da96f655deb0a4167718775b9ed03af7baca8d545913f13a25e3a56c18b8901"
        artifact_pdf_hash: "sha256:7da96f655deb0a4167718775b9ed03af7baca8d545913f13a25e3a56c18b8901"
        "###);
        check_bundle_facts!("visualize", "stroke_3", @r###"
        ---
        name: visualize/stroke_3.typ
        origin_pdf_hash: "sha256:4ed2e2f053c3bb53e9a698425fe7be8f37ee6804bcce17fa8e169d7ae42a232d"
        artifact_pdf_hash: "sha256:4ed2e2f053c3bb53e9a698425fe7be8f37ee6804bcce17fa8e169d7ae42a232d"
        "###);
        check_bundle_facts!("visualize", "stroke_4", @r###"
        ---
        name: visualize/stroke_4.typ
        origin_pdf_hash: "sha256:8eb4e3ef1bf6098fe1fde4172e5afc89a91d9d25ac7b0eca169af3da1eae2f45"
        artifact_pdf_hash: "sha256:8eb4e3ef1bf6098fe1fde4172e5afc89a91d9d25ac7b0eca169af3da1eae2f45"
        "###);
        check_bundle_facts!("visualize", "stroke_5", @r###"
        ---
        name: visualize/stroke_5.typ
        origin_pdf_hash: "sha256:3c107e3bea0b5ecd2bb3148f30d443c3bbedf45f1c6da8bad81d605cd317747c"
        artifact_pdf_hash: "sha256:3c107e3bea0b5ecd2bb3148f30d443c3bbedf45f1c6da8bad81d605cd317747c"
        "###);
        check_bundle_facts!("visualize", "stroke_6", @r###"
        ---
        name: visualize/stroke_6.typ
        origin_pdf_hash: "sha256:0fee152787b0234cfcc767c110eae8197866bbb077f1baff5b1e7f147d5d5fe1"
        artifact_pdf_hash: "sha256:0fee152787b0234cfcc767c110eae8197866bbb077f1baff5b1e7f147d5d5fe1"
        "###);

        check_bundle_facts!("text", "chinese", @r###"
        ---
        name: text/chinese.typ
        origin_pdf_hash: "sha256:74cd5fa5938b57ed100da382567460a98662ef4de72eab24894d529ebca5151d"
        artifact_pdf_hash: "sha256:74cd5fa5938b57ed100da382567460a98662ef4de72eab24894d529ebca5151d"
        "###);

        check_bundle_facts!("text", "deco_1", @r###"
        ---
        name: text/deco_1.typ
        origin_pdf_hash: "sha256:a9e03a591e5b930da0397a16e6a21d77973a93f6556f85e4a1bad66a4a449538"
        artifact_pdf_hash: "sha256:a9e03a591e5b930da0397a16e6a21d77973a93f6556f85e4a1bad66a4a449538"
        "###);
        // todo: figure out why rgba does not work
        check_bundle_facts!("text", "deco_2", @r###"
        ---
        name: text/deco_2.typ
        origin_pdf_hash: "sha256:abd47bb191f85eb0343cd9f2fde209b879362cfe6c9a35e48c1807e08385caa3"
        artifact_pdf_hash: "sha256:abd47bb191f85eb0343cd9f2fde209b879362cfe6c9a35e48c1807e08385caa3"
        "###);
        check_bundle_facts!("text", "deco_3", @r###"
        ---
        name: text/deco_3.typ
        origin_pdf_hash: "sha256:6dcd3913deed9aec0f532855932f90d53ccfc2697e3b9bf4f429a8b34c20da5c"
        artifact_pdf_hash: "sha256:6dcd3913deed9aec0f532855932f90d53ccfc2697e3b9bf4f429a8b34c20da5c"
        "###);

        check_bundle_facts!("text", "emoji_1", @r###"
        ---
        name: text/emoji_1.typ
        origin_pdf_hash: "sha256:81b7ddbedf14d5c832256571591480f3522a043769f9e26c3a27c3432987e350"
        artifact_pdf_hash: "sha256:81b7ddbedf14d5c832256571591480f3522a043769f9e26c3a27c3432987e350"
        "###);
        check_bundle_facts!("text", "emoji_2", @r###"
        ---
        name: text/emoji_2.typ
        origin_pdf_hash: "sha256:38a9b8adfcc095b848dfa71aee1caa1154a1029493c8ed540e5fb04802eaf709"
        artifact_pdf_hash: "sha256:38a9b8adfcc095b848dfa71aee1caa1154a1029493c8ed540e5fb04802eaf709"
        "###);
        // todo: typst cannot pass visualize/stroke_6 test.

        fn bundle_to_facts(name: String, bundle: &ArtifactBundle) -> Facts {
            // todo: pdf export by svg?
            // let json_artifact = std::fs::read(&bundle.json).unwrap();
            // let json_artifact =
            // serde_json::from_slice::<typst_ts_core::Artifact>(&json_artifact)
            //     .expect("failed to deserialize json artifact");

            // let doc = json_artifact.to_document(&bundle.driver.world().font_resolver);

            // let pdf_path = bundle.pdf.with_extension("artifact.pdf");
            // std::fs::write(pdf_path, &pdf_doc).unwrap();

            let origin_doc = std::fs::read(&bundle.pdf).unwrap();

            let artifact_pdf_hash = hash_bytes(&origin_doc);
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
                let data_content_hash = test_point
                    .meta
                    .remove("data_content_hash")
                    .expect("data_content_hash not found");
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
                    "\n::group::Snapshot testing Failure (Browser Canvas Rendering)\nsnapshot does not match the older one\nTestPointName: {}\nDataContent: {}\nTextContent: {}\nDataContentHash: {}\n::endgroup::",
                    test_point.name,
                    data_content,
                    text_content,
                    data_content_hash,
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
        name: layout_clip_00_artifact_ir
        data_content_phash: "phash-gradient:AAAAADAAIEAAADUAJLcBLPUFKPUFNLUBALUAILUABLUALLUAMLUAKLUACEAAADIACAQBAAMCJOMFKOUFKOUFAAAAAIEAAAAA"
        text_content_hash: "sha256:76808cf8c41bd86329766e35346c3a987df61dabd5036ce2e8871b6bf014c1a7"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_01_artifact_ir
        data_content_phash: "phash-gradient:FBIAAAAANFsAFBMAFBMABBIAvDYAJFcAEBsALAEAiBMAmAMAAAAAABEAAAAAqBgAFFMAFDMAJBEAnBYApBYApF4AAAAAABQA"
        text_content_hash: "sha256:93cc86f5b4466bef3b400ed07a9fd183972793729ccd319a361869163f77c37c"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_02_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAABIABMAACTO4FWKkFcJMFULIFTJIFYJIFAJIFAAAAAJIBTAACTIQFUJMFcJMFWJIFQO0BoAAAAIAAAAAAAAAA"
        text_content_hash: "sha256:92325a7e28f6b65691ffe3f066601855ace6258c2db55423c37ab54bb8af33d0"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_03_artifact_ir
        data_content_phash: "phash-gradient:EAAARAAAbAEAcAJEAQAANJPOFCnelCHHLGnSnGHckJbQAAAAAAAABKXh7Grc5ErOJCXAFGvWKJVhAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:5ac4c18951ce4045326e846cd054bb7d648b3c478d590a5dcd7194544175db18"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list-marker_00_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAoAAApAAApAAAoAAAIAAAAAAAAAAAMAAAsAAAtAAAtAAAsAAANAAAAAAAEAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:e8bc97ec7d047e00f51b485d8f9eaf83e254be1e2a5886ca3a43192ba7dd4e70"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list-marker_01_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAIAAASAAAoAAApAAApAAAIAAARAAAIAAAwAAA0AIA1AIAwAAAAAIAwAAAAAoAQAMAQAMAAAsAgAQAAAIAAAAAAAAA"
        text_content_hash: "sha256:733cbf09c606be8205623391a75cae77b53089ddcdb79e1ddc7520b0aebfdf03"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list-marker_02_artifact_ir
        data_content_phash: "phash-gradient:EAAABAAAMAAAtAAAtAAAWAAANAAAtAAAAAAAxAIA0AAAwAAAwAAA0AIA0AIAAAAAAAUAQAEABAMAUAAAVAEAVAAAIAAAFAAA"
        text_content_hash: "sha256:e304f11bb3c774bcd260c028beab517f7b383299c1f4d9520f03343c2d70b464"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list-marker_03_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAkAAAAAEJsBEpkAUuoFRLIFYrIFUKwBAAwAUIAAIFEAIBEAwNIA1GQAknwAkEgAAAAAgAgAAAAAAAAAAAAA"
        text_content_hash: "sha256:87e6e1cd2b4ad9e48ac686ebc22c9362f7770ff274d49702c13b15d072e9fbbb"
        "###);
        // ok empty page, compile error
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list-marker_04_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_00_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAASGgAAIIAzGgBzWgBYWsAuGsBgNkCVPMACLIAAEAAkDIAwLIAlHoAkHQAmDUAFD0AAE0AEC0AAAAAAAAAAAAA"
        text_content_hash: "sha256:f1b9f63c42525a7552deae670ee0fb0aedc4781c1208cf04988c6009d7a11007"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_01_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAIAgAkAICMAII8AgD0BkPkMEPMTkNkvSBtbyFxeyM8siI0vhIstBIYvCJoXEMAXQIAWAAEXAAQLABALAEACAIAAAAAA"
        text_content_hash: "sha256:4a7081123998d824b27aada4783ff7493a3286a65519e4b65d294c0e8ed5ebcf"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_02_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAABAAAIAAADAEAPAQA8BEAZCcA7FkAaFsAZFwAbFkANBsANB8ALB8APC8AHC8AEC4AQi0AICwAACwABiwAECwAgAgA"
        text_content_hash: "sha256:91e3c629537ebadd6dfc515c298206a18f701b0823f31b607c8cfe86be5e7ac3"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_03_artifact_ir
        data_content_phash: "phash-gradient:AAAAABgAACAACFgACFoACFoACFoACFoCCBoASFoDCEALKEEDABnjICXkACDqACToACToACToACToACTgAAgAAABAAAAAAAAA"
        text_content_hash: "sha256:8c226be77aa543c70f8b572e14ad16777fcfda2c6bada29c5b099a82817a9f77"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: math_main_artifact_ir
        data_content_phash: "phash-gradient:AAAAgNwAAMQAmAYA2M4AAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:bba6398b530a028e28ad963843562356b21105ce598af1a07c60495e56928acb"
        "###);
        // todo: emoji change
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: math_undergradmath_artifact_ir
        // data_content_phash:
        // "phash-gradient:YMrt5IztbCb1OIf2MMdmmEbmmEzkUIjjcI7JsK/
        // edMYnZIzoqI7ceIfJNI94LIs2BBgAAAAAAAAAAAAAAAAAAAAAAAAAgBgA"
        // text_content_hash:
        // "sha256:f171fe5c31efece4159215ee0f7984fa9d36cab8c9c4ab3d6035c251a9099c14"
        // "###);
        // todo: the size of cjk font file is quite big
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_chinese_artifact_ir
        // data_content_phash:
        // "phash-gradient:
        // KKprrKlq6Kxm0KTmZKpaZIrbNGI0pNI0tZI1qBI1rDy1bIpqLJpjqFU2qFUlVFS1hIkalIkasGoasKpStWhmpGhmiCoGqZYE"
        // text_content_hash:
        // "sha256:08633df6b8b06027fee154dccd3d530fd53db36851c597621c8e8b65e52b028b"
        // "###);
        // text_content_hash:
        // "sha256:08633df6b8b06027fee154dccd3d530fd53db36851c597621c8e8b65e52b028b"
        // "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_00_artifact_ir
        data_content_phash: "phash-gradient:AAAArDQakCRbJNJIbJJsSLV4AAAA5Gog0GwUDAkApCwVoMQ0UCwAkGoRgAQAGFg0kNiykJQwjCocREldMJkZtAEAkAUIAAAA"
        text_content_hash: "sha256:7fa19142cc94829430e2cf433cd94204da0d640ab6f0a6c032b47eb6761e5c8d"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_01_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAghAAAYCRihABklCVlaSZlIG9kAAhgwGQKSAMpwEo6yNxawA4byAABAEMSUA0B1N0AIPECYOkCCGMBAJAAACEAAAAA"
        text_content_hash: "sha256:f3297a47cfe8c258fdc15e330f1f6100f37274e5de19ed85e1cf442c63964f9f"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: text_deco_02_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAMABAcA4BYAaAcA0AYAyAYANBcAMAcAyBYACAYAAAAAAAQAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:102a67f29820229d33203cba9598ab5007b3ce0a80ea5774722a70531dd6545c"
        "###);
        // still inconsisistent
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_emoji_1_artifact_ir
        // data_content_phash:
        // "phash-gradient:
        // AABAwKdFQLZFyIdFwINFAABCAABAQIBBSKBFSKBFQKBBAABAgMFBAAZAAPZBANFFANJCAABBAIBBAABAQCJLCCdLCCZLANhD"
        // text_content_hash:
        // "sha256:e96d18327a60513e2375c2dfa12d17872c97304df451630781965a6ae8031b45"
        // "###);
        // text_content_hash:
        // "sha256:e96d18327a60513e2375c2dfa12d17872c97304df451630781965a6ae8031b45"
        // "###);
        // todo: we don't compile it with emoji fonts
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_emoji_01_artifact_ir
        // data_content_phash:
        // "phash-gradient:
        // AAAAAAAAAAAAAAAAAAAAAAAAKAAAAAAAKAAAeAAAuAAAuAAAqQAAKQAAbAEAZAEAYAEAYAAAAAAAIAAAAAAAAAAAAAAAAAAA"
        // text_content_hash:
        // "sha256:4317f46900063f5e598a07d44c10a38d2947205168be6442ca451daa186371a2"
        // "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_00_artifact_ir
        data_content_phash: "phash-gradient:AAAAVAAAAAEA1IIA1JACpIECgeQCAmAAgAAACAAAMgAATAAAMQEA4gQAiBMAIGYAwJgBAGECAMQFABAFAEACAAABAAAAAAAA"
        text_content_hash: "sha256:7a7127a2ee0463c8ca6a429f6d8e04bfb7db288f69f9329cd3e6bf94a36ea1a8"
        "###);
        // ok empty page, compile error
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_01_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAASBQFtGwSOo4jWpYlNE0TGFUHSDIMtGwSOo4jWpYlNE0TGFUHSDIMtGwSOo4jWpYlNE0TGFUHwDAMEAQCAAAAAAAA"
        text_content_hash: "sha256:0adb47d06ff5feb21ec0f42160dc35593affcde6a79680c5b14528b16d6251ef"
        "###);
        // ok empty page, compile error
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_02_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        // ok empty page, compile error
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_03_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_path_00_artifact_ir
        data_content_phash: "phash-gradient:BAjAAhOgAhCAwpOgolOaylWs0pK0ylWsolOewpOgAhCAAhGgYhCgAhCAalap4pG0opesIleqojGb4jSZalGgAhCAahCgBAjA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_path_01_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_path_02_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_path_03_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_polygon_00_artifact_ir
        data_content_phash: "phash-gradient:AAAAYAYAgAMAgB8AAB4AgEEEwAcAAPYPwPAP4AAAAMEeAMMeEPgAAPAHpMcGJPAACHAvAIcHAIofAPAfAPAvAMoPBNIvBMIP"
        text_content_hash: "sha256:646d73bed3a7ae35a06c81746bca783add37393454634484d4bed6315602a0a2"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_polygon_01_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-aspect_00_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAgABFoABDAAFBAAFBAAFBAQFBAAFBAZlBC4lBK1lLU0lDX0FLTclLTUFLTUFLT0FLTUBDDMBLDcAAhjAAAAAAAA"
        text_content_hash: "sha256:796d70163b44634c3948e96fee2fe1d3b712f4eeb6b1061129489ec0ff47adbe"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-aspect_01_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAQAAAAAiA4AJK0B3AwCkM8FlCULJBQCFJkWFIkUFAkUFMkNFMkNRIkFFAkUxNkUxJkWJDQKVC0DKM4EABACCIYAAAAA"
        text_content_hash: "sha256:c251d63de7b61c52f0d01160f10ee1622e550afe29ed4510f0907e779f75ba84"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-aspect_02_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAYAMAgAQAaAsAJBcAhhQAxBQAxAwAxAwAxC0AxC0AxAwAxAwAxBQAhhQAJAcAaAsAgAQAYAMAAAAAAAAAAAAA"
        text_content_hash: "sha256:9a77a41dcb9d094bd5f8835090e1fff669f88ad82df3c46b1becd580a4626e1a"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-aspect_03_artifact_ir
        data_content_phash: "phash-gradient:AFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoAAFoA"
        text_content_hash: "sha256:9a77a41dcb9d094bd5f8835090e1fff669f88ad82df3c46b1becd580a4626e1a"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-aspect_04_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAKAEAEgIAKQUAKQsA1AoA1AoA1AoA1AoAKQsAKQUAEgIAKAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:9a77a41dcb9d094bd5f8835090e1fff669f88ad82df3c46b1becd580a4626e1a"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-aspect_05_artifact_ir
        data_content_phash: "phash-gradient:AAAAEBgAKTIACTAAKTIAKTAAKTAAKTIACTAAKTIAEBgAACAAAAAAEAAwiURpCQBgiUZpiVZtiVpqiVpriZZlKQBoKRNpSQpo"
        text_content_hash: "sha256:74543202540c610179ecf8a6be931bbe1ee7ba28b96aa7cf63830c5239b0301d"
        "###);
        // ok empty page, compile error
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-aspect_06_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-circle_00_artifact_ir
        data_content_phash: "phash-gradient:AAAAACAAAEQAALIAAGkBUMwCpLQBYiYF2BoFjXoDJHMDNGMDFGMDFFMDFEsDNAsFpCYFrbQBicwCUmkBJLIAiEQAICAAAAAA"
        text_content_hash: "sha256:767003fd533a51edac086f265dd179363a7590ddcb0638ce20b1e6607b5b52af"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-circle_01_artifact_ir
        data_content_phash: "phash-gradient:OKUBgI8BQD4AfLQSgR8AxEYasNQbBL8AAf8C6cgCgfgqAD8AvE5pBPhpwOQG2eYsWacsAccHAPwBNFsO0VYP3BgA3BgAwQYA"
        text_content_hash: "sha256:7755eebfec95303db7912c8e2a8de7681b777624e920a0d1b33fb1712f4e00a5"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-circle_02_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAQAAAAACAYAiBYA6BYAmBYAnBYADBYALBYALBYALBYALBYADBYAnBYAmBYA6BYAiBYACAYAAAAAAAQAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-circle_03_artifact_ir
        data_content_phash: "phash-gradient:AAAAAABAAAAACADoCIDkCEPsiCz6SBz1MJrwMJXkKJNraBNrSBNpKJNrMJXkMJrwSBz1iCz6CEPsCIDkCADoAAAAAABAAAAA"
        text_content_hash: "sha256:785cae5ddc3bd458b0ba93329145e75afaecff8eb5e61395ab33fe6da3705dc8"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape-circle_04_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_00_artifact_ir
        data_content_phash: "phash-gradient:AEAAAAAACGgACGAAAAAAAAAAAGAACGgBAGAAAAAAAIAACGgBCGgBAIAAAGAAAAAAJGwAAAAAAEAAAQAABOgABOgAAQAABGAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_01_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAEAAAIAApHQBpHQAAAAAAAAAAAAAAEAAAIAApHQBpHQBAIAAAEAAAAAAACAAAAAACGAACGgBAIAAAGAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_02_artifact_ir
        data_content_phash: "phash-gradient:AAAAYAAAZAEARAEARAEARAEAZAEAIAAAAAAAZAAARAEARAEARAEARAEAbAEAgQAAaAEAZAEARAEARAEARAEAZAEAaAEAgQAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_03_artifact_ir
        data_content_phash: "phash-gradient:ACAAAAAAAGAAAGAAAAAAAAAAAAAAlHoAAAAAACAAAAAASGkASGkAAAAASCkAACAACGgBACAAEEQAAAAAOE4AOE4AAAAAIEQA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_04_artifact_ir
        data_content_phash: "phash-gradient:AABoAIRgIAGkpJS1tJSxspYSMsZaWMZYGcNYGeNcnHMYnHMsLGMsKWUsKWMtGWMsGMNYlNZatPZUMMa4MIaxpBAlEEJKAAAh"
        text_content_hash: "sha256:d3ef8febcdcfc79f21cbfcd4d61dcd5b8cfc7bd303fcbecb5e0467cef81e833f"
        "###);
        // ok empty page, compile error
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_05_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        // ok empty page, compile error
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_06_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:0b98b0a2660a801a610067cf0ad5726a8f2012d55d4f6174fec469d7c49a49eb"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_07_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAACAAAIAAAWAAAWAAAAAAAWAAAGAAAAAAACAAAAAAAgAAAmgAAmAAAIAAAmgAAmAAAIAAAkAAAAAAAAAAA"
        text_content_hash: "sha256:e51ffb4d7aa2ed1e98d8a3222d8648eabffabb21e11769aa0a0a35ca667bf767"
        "###);

        let done = test_point_iter.next();
        if done.is_some() {
            panic!("test_point_iter is not empty: {}", done.unwrap().name);
        }

        Ok(())
    }
}
