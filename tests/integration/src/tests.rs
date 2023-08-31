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
            // let pdf_doc = typst::export::pdf(&doc);

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
        name: layout_clip_1_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAACAACAGkELOkLaOkKKOkKAG0BKGkBIGkBBGkBPGkBIGkBKGkBCIAABGIASAACACcELIcKKMsKaKsKJEACAAgAAAAA"
        text_content_hash: "sha256:320577a48dd36fcf697605bb46b64c44ed5e6a39eed6a6e06813f64e9d73e70f"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_1_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAACAACAGkELOkLaOkKKOkKAG0BKGkBIGkBBGkBPGkBIGkBKGkBCIAABGIASAACACcELIcKKMsKaKsKJEACAAgAAAAA"
        text_content_hash: "sha256:320577a48dd36fcf697605bb46b64c44ed5e6a39eed6a6e06813f64e9d73e70f"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_2_artifact_ir
        data_content_phash: "phash-gradient:BAAAyBEANLYAFGcAtG8AlmQANGwAZC0AUDIAbAMAWAsAmAMAAAAASBMAAAAAyDEANCYAlGcAJGYAlGwAdG0AZK0AABAAIAUA"
        text_content_hash: "sha256:f5db2f803136c1ae0a3a83f4cf86a39e2c56efc0c33cd6ad85024c46080eed79"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_2_artifact_json
        data_content_phash: "phash-gradient:BAAAyBEANLYAFGcAtG8AlmQANGwAZC0AUDIAbAMAWAsAmAMAAAAASBMAAAAAyDEANCYAlGcAJGYAlGwAdG0AZK0AABAAIAUA"
        text_content_hash: "sha256:f5db2f803136c1ae0a3a83f4cf86a39e2c56efc0c33cd6ad85024c46080eed79"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_3_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAABAAAAIACjAAATNoDUGIGUCYCWGYKQCQLoGQLAGQLAAAAACADsAAATNoDVHIGUCYGcCYCXNoDQAAANAACAAAAAAAA"
        text_content_hash: "sha256:1d003760abd6ef9775b6e7ff2941272dcf5b5d4467097ab7ed6edc8cb5660d04"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_3_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAABAAAAIACjAAATNoDUGIGUCYCWGYKQCQLoGQLAGQLAAAAACADsAAATNoDVHIGUCYGcCYCXNoDQAAANAACAAAAAAAA"
        text_content_hash: "sha256:1d003760abd6ef9775b6e7ff2941272dcf5b5d4467097ab7ed6edc8cb5660d04"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_4_artifact_ir
        data_content_phash: "phash-gradient:AAAABAAAIAAAjAAAzAAAYAAAaAAAUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:ada49b7111c4303c8768eeb0ecb917e92486adc4e0cd27c39ffc735138a05eae"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_clip_4_artifact_json
        data_content_phash: "phash-gradient:AAAABAAAIAAAjAAAzAAAYAAAaAAAUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:ada49b7111c4303c8768eeb0ecb917e92486adc4e0cd27c39ffc735138a05eae"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list_marker_1_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAIAAAoAAApAAAJAAAYAEAZAAAAAAAAAAAMAAApAAAoAAAJAAAJAAAsAAASAAAIAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:d3623a444f2ea8f2eec30c44d9df3f68659f5bf14d0e0f3a90b087c033ce6052"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list_marker_1_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAIAAAoAAApAAAJAAAYAEAZAAAAAAAAAAAMAAApAAAoAAAJAAAJAAAsAAASAAAIAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:d3623a444f2ea8f2eec30c44d9df3f68659f5bf14d0e0f3a90b087c033ce6052"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list_marker_2_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAIAAABAAAoAAApAAAZAAAYAAAiAAAIAAAyAIAkAIAlAAAkAIAIAIAgAAAAAIAQAMAUAMAQAoAAAAAAAEAAAAAAAAA"
        text_content_hash: "sha256:3423834c3234ee14869f398e04d67fcd555a8e6f45bfd2f6f131dbeb2876f39f"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list_marker_2_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAIAAABAAAoAAApAAAZAAAYAAAiAAAIAAAyAIAkAIAlAAAkAIAIAIAgAAAAAIAQAMAUAMAQAoAAAAAAAEAAAAAAAAA"
        text_content_hash: "sha256:3423834c3234ee14869f398e04d67fcd555a8e6f45bfd2f6f131dbeb2876f39f"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list_marker_3_artifact_ir
        data_content_phash: "phash-gradient:IAAASAAAtAAAtAAAJAAAEAAAtAAAtAAAAAAAlAAA0AAAgAAAwAAAkAEAkAEAAAAAUAsAUAsABAIAsAAANAIAVAAAAAAAFAAA"
        text_content_hash: "sha256:7c62afa2f5936df2a497e6adc5c53d289fa7d5a5d362f1da8f5bbf4aa7d3be88"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list_marker_3_artifact_json
        data_content_phash: "phash-gradient:IAAASAAAtAAAtAAAJAAAEAAAtAAAtAAAAAAAlAAA0AAAgAAAwAAAkAEAkAEAAAAAUAsAUAsABAIAsAAANAIAVAAAAAAAFAAA"
        text_content_hash: "sha256:7c62afa2f5936df2a497e6adc5c53d289fa7d5a5d362f1da8f5bbf4aa7d3be88"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list_marker_4_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAEBIAgAgAMDIDMDIDlNIG5OQG4KIG0NgCAAgAkEACaKUBaCMCQKUFlJUFkOkBkJUBAAAAgJAAAAAAAAAAAAAA"
        text_content_hash: "sha256:a75dd2a003742b03b7e654d5fc84a2b0a4faadbe35f4443073ebca442864c7cf"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_list_marker_4_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAEBIAgAgAMDIDMDIDlNIG5OQG4KIG0NgCAAgAkEACaKUBaCMCQKUFlJUFkOkBkJUBAAAAgJAAAAAAAAAAAAAA"
        text_content_hash: "sha256:a75dd2a003742b03b7e654d5fc84a2b0a4faadbe35f4443073ebca442864c7cf"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_1_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAqAIRQAEAuAY5uhZbUjVaoD1aqDRyyDZuEEAmgIAQSGwMAGYqiLQq0JQO0LQs2JQNAJABiJoNAAAAAAAAAAAA"
        text_content_hash: "sha256:09474ddd5218dbce7c2b3f535ba83a1088f2f39ea1c8cdc6fd1a66d72497731e"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_1_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAqAIRQAEAuAY5uhZbUjVaoD1aqDRyyDZuEEAmgIAQSGwMAGYqiLQq0JQO0LQs2JQNAJABiJoNAAAAAAAAAAAA"
        text_content_hash: "sha256:09474ddd5218dbce7c2b3f535ba83a1088f2f39ea1c8cdc6fd1a66d72497731e"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_2_artifact_ir
        data_content_phash: "phash-gradient:AABAAAJAgAhAgCNAEI9AkD1CkPlIAPsDyNlPSBlbyFxchM9ZhI1bhItvJIdvAJpPAMBPIIBWAAFWAARWABBCAEBCAABBAABA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_2_artifact_json
        data_content_phash: "phash-gradient:AABAAAJAgAhAgCNAEI9AkD1CkPlIAPsDyNlPSBlbyFxchM9ZhI1bhItvJIdvAJpPAMBPIIBWAAFWAARWABBCAEBCAABBAABA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_3_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAABAAAIAAAjAAAPAQA8BEAZCcA7FkAaFsAZFwAbFkANBsANB8ALB8APC8AHC8AEC4AUi0AIC0AACwABiwAAAwAgAkA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_3_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAABAAAIAAAjAAAPAQA8BEAZCcA7FkAaFsAZFwAbFkANBsANB8ALB8APC8AHC8AEC4AUi0AIC0AACwABiwAAAwAgAkA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_4_artifact_ir
        data_content_phash: "phash-gradient:AAAAABgAACAACFgACFoACFoACFoACFoBCBoASFoDCFhLKEUDAFnrICXsACDiACToACToACToACToACToACBgAAAAAABAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: layout_transform_4_artifact_json
        data_content_phash: "phash-gradient:AAAAABgAACAACFgACFoACFoACFoACFoBCBoASFoDCFhLKEUDAFnrICXsACDiACToACToACToACToACToACBgAAAAAABAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);

        check_canvas_render_test_point!(@r###"
        ---
        name: math_main_artifact_ir
        data_content_phash: "phash-gradient:AAAAgNwAAMQAmAYA2M4AAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:f29154e604d9bc1b6f215b5ad2e28ac33103f04d3a58a92e8c3fa4c60ca361b7"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: math_main_artifact_json
        data_content_phash: "phash-gradient:AAAAgNwAAMQAmAYA2M4AAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:f29154e604d9bc1b6f215b5ad2e28ac33103f04d3a58a92e8c3fa4c60ca361b7"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: math_undergradmath_artifact_ir
        data_content_phash: "phash-gradient:YGMAQMtNZIzN7CXxZujtZKoW5AT1cGbRDG9mmI1ilIQ2MINiMMNmOEdmMMdkOMPmmGbmmEZkmYbW2GzONInhUAxjSY43KAYA"
        text_content_hash: "sha256:f3d341f4cdd35ca3fa6908adfb1f90b8c4ef907c552a33899138844db4e894b6"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: math_undergradmath_artifact_json
        data_content_phash: "phash-gradient:YGMAQMtNZIzN7CXxZujtZKoW5AT1cGbRDG9mmI1ilIQ2MINiMMNmOEdmMMdkOMPmmGbmmEZkmYbW2GzONInhUAxjSY43KAYA"
        text_content_hash: "sha256:f3d341f4cdd35ca3fa6908adfb1f90b8c4ef907c552a33899138844db4e894b6"
        "###);
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
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_chinese_artifact_json
        // data_content_phash:
        // "phash-gradient:
        // KKprrKlq6Kxm0KTmZKpaZIrbNGI0pNI0tZI1qBI1rDy1bIpqLJpjqFU2qFUlVFS1hIkalIkasGoasKpStWhmpGhmiCoGqZYE"
        // text_content_hash:
        // "sha256:08633df6b8b06027fee154dccd3d530fd53db36851c597621c8e8b65e52b028b"
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
        // data_content_phash:
        // "phash-gradient:
        // AABAwKdFQLZFyIdFwINFAABCAABAQIBBSKBFSKBFQKBBAABAgMFBAAZAAPZBANFFANJCAABBAIBBAABAQCJLCCdLCCZLANhD"
        // text_content_hash:
        // "sha256:e96d18327a60513e2375c2dfa12d17872c97304df451630781965a6ae8031b45"
        // "###);
        // check_canvas_render_test_point!(@r###"
        // ---
        // name: text_emoji_1_artifact_json
        // data_content_phash:
        // "phash-gradient:
        // AABAwKdFQLZFyIdFwINFAABCAABAQIBBSKBFSKBFQKBBAABAgMFBAAZAAPZBANFFANJCAABBAIBBAABAQCJLCCdLCCZLANhD"
        // text_content_hash:
        // "sha256:e96d18327a60513e2375c2dfa12d17872c97304df451630781965a6ae8031b45"
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
        text_content_hash: "sha256:dd4e9660af4ec19d152b728b986c191479b340adf1c2a9223a1631beaa0bd22d"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_1_artifact_json
        data_content_phash: "phash-gradient:JAEAAAIAJAEEJIcURAYEQRMXskAHBAAABAACCQAANgAATAAAMQEA4gYAiAkAICYAQMwAADEBAMQCAIgFACABAEAAAAABAAAA"
        text_content_hash: "sha256:dd4e9660af4ec19d152b728b986c191479b340adf1c2a9223a1631beaa0bd22d"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_2_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAgAAJIBgE0CgE0CgE0CgE0CgE0CgE0CgE0CgE0CABIBAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_line_2_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAAAAAAAAAAAAgAAJIBgE0CgE0CgE0CgE0CgE0CgE0CgE0CgE0CABIBAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_path_1_artifact_ir
        data_content_phash: "phash-gradient:AAAgAAAACBBgKRBuDRBADTNNjTNGjVVCjTNGTTNNDRBACTNtqVBqDRBAzVFKyVdGSTZFybNNzZVtTRBAKRFpSBBgAAAAAAAg"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_path_1_artifact_json
        data_content_phash: "phash-gradient:AAAgAAAACBBgKRBuDRBADTNNjTNGjVVCjTNGTTNNDRBACTNtqVBqDRBAzVFKyVdGSTZFybNNzZVtTRBAKRFpSBBgAAAAAAAg"
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
        text_content_hash: "sha256:24fc35fa6d7a18dc6a5578cf44456eb696b77bffcbc04f2d3048e34289dd364d"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_aspect_1_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAgABFoABjAABhAABhAABhAABhAwBhBCBhCwlhKpljQtljXsBjTZhjSNBjStBjSlFjSlBjCMFjSNBEjSAQAAAAjC"
        text_content_hash: "sha256:24fc35fa6d7a18dc6a5578cf44456eb696b77bffcbc04f2d3048e34289dd364d"
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
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_circle_1_artifact_ir
        data_content_phash: "phash-gradient:AAAAACAAAIQAAGIBANkCII0BiDQFZEYDUloLiXIKrXMKNEMKFEMKFBsKFEsKFEsLtEYDrSQFiYwB0tgCJCIBiIgAAAAAAAAA"
        text_content_hash: "sha256:3dd31edd9b525f0bb9ecf8f888bcb6b0e923a0a3a664b72fb614ed9c3d9e7851"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_circle_1_artifact_json
        data_content_phash: "phash-gradient:AAAAACAAAIQAAGIBANkCII0BiDQFZEYDUloLiXIKrXMKNEMKFEMKFBsKFEsKFEsLtEYDrSQFiYwB0tgCJCIBiIgAAAAAAAAA"
        text_content_hash: "sha256:3dd31edd9b525f0bb9ecf8f888bcb6b0e923a0a3a664b72fb614ed9c3d9e7851"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_circle_2_artifact_ir
        data_content_phash: "phash-gradient:eEsDAR8DQHwAVGgpAT8AhIxqsChrAH4AAfgFyZEBAfwBAH4ErJgFSBgIAPwT2eQuWOMsQYcOAvgDdLUcsa8cnBEAnDEAkQwA"
        text_content_hash: "sha256:19e55365f6d562d1e7c5cccf1836b4101e49ca12a4df2bd5ff28869e6a59afce"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_circle_2_artifact_json
        data_content_phash: "phash-gradient:eEsDAR8DQHwAVGgpAT8AhIxqsChrAH4AAfgFyZEBAfwBAH4ErJgFSBgIAPwT2eQuWOMsQYcOAvgDdLUcsa8cnBEAnDEAkQwA"
        text_content_hash: "sha256:19e55365f6d562d1e7c5cccf1836b4101e49ca12a4df2bd5ff28869e6a59afce"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_circle_3_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAQAAAAACAYAiBYA6BYAmBYAnBYADBYALBYALBYALBYALBYADBYAnBYAmBYA6BYAiBYACAYAAAAAAAQAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_circle_3_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAQAAAAACAYAiBYA6BYAmBYAnBYADBYALBYALBYALBYALBYADBYAnBYAmBYA6BYAiBYACAYAAAAAAAQAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_circle_4_artifact_ir
        data_content_phash: "phash-gradient:AAAAAABAAAAACADoCIDkCEPsiCz6SBz1MJrwMJXsKJNraBNhSBNpaJNrMJXkMJrySBz1iCz6CEPsCIDkCADoAAAAAABAAAAA"
        text_content_hash: "sha256:31d2cf7d42f57b7401ffcd55203058d910afbdd50029cb0d7ad4d4139e7f29a1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_shape_circle_4_artifact_json
        data_content_phash: "phash-gradient:AAAAAABAAAAACADoCIDkCEPsiCz6SBz1MJrwMJXsKJNraBNhSBNpaJNrMJXkMJrySBz1iCz6CEPsCIDkCADoAAAAAABAAAAA"
        text_content_hash: "sha256:31d2cf7d42f57b7401ffcd55203058d910afbdd50029cb0d7ad4d4139e7f29a1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_1_artifact_ir
        data_content_phash: "phash-gradient:AEAAAAAACGgBAGAAAAAAAAAACGAACGgBAAAAAGAAAIAACGgBCGABAIAAAGAAAAAABGgAACAAAEAAAQAABOgABOgAAAAAAEAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_1_artifact_json
        data_content_phash: "phash-gradient:AEAAAAAACGgBAGAAAAAAAAAACGAACGgBAAAAAGAAAIAACGgBCGABAIAAAGAAAAAABGgAACAAAEAAAQAABOgABOgAAAAAAEAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_2_artifact_ir
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAApHQBpFQAAAAAAAAAAAAAAEAAAIAApHQBpHQBAIAAAEAAAAAAAAAAAAAAAGAACGgBAAAAAGAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_2_artifact_json
        data_content_phash: "phash-gradient:AAAAAAAAAAAAAAAApHQBpFQAAAAAAAAAAAAAAEAAAIAApHQBpHQBAIAAAEAAAAAAAAAAAAAAAGAACGgBAAAAAGAAAAAAAAAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_3_artifact_ir
        data_content_phash: "phash-gradient:AAAAaAAAZAEARAEARAEARAEAbAEAAAAAAAAAZAAARAEARAEARAEARAEAaAAAgAAAaAAAZAEARAEARAEARAEAZAMAaAAAgwAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_3_artifact_json
        data_content_phash: "phash-gradient:AAAAaAAAZAEARAEARAEARAEAbAEAAAAAAAAAZAAARAEARAEARAEARAEAaAAAgAAAaAAAZAEARAEARAEARAEAZAMAaAAAgwAA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_4_artifact_ir
        data_content_phash: "phash-gradient:ACAAAAAACGAAAGAAAAAAAAAAAGAAlHoAAAAAAEAAAAAASGkASGkAAAAAACAAAAAACGABAGAAAAAAAAAAOE4AOE4AAAAAMEYA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_4_artifact_json
        data_content_phash: "phash-gradient:ACAAAAAACGAAAGAAAAAAAAAAAGAAlHoAAAAAAEAAAAAASGkASGkAAAAAACAAAAAACGABAGAAAAAAAAAAOE4AOE4AAAAAMEYA"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_5_artifact_ir
        data_content_phash: "phash-gradient:AABAAARJIEBqpKxltKylspS1MpayWMayGcY4GcdYnMNcHONcLGNZKWspKWtZGUNZGMJYlKa0tKe0MIaxMIRhpCBnEEKQAABB"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_5_artifact_json
        data_content_phash: "phash-gradient:AABAAARJIEBqpKxltKylspS1MpayWMayGcY4GcdYnMNcHONcLGNZKWspKWtZGUNZGMJYlKa0tKe0MIaxMIRhpCBnEEKQAABB"
        text_content_hash: "sha256:7697c705e134fe39094c2ad9d6076210e20079cb32d7479079961e97237081d1"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_6_artifact_ir
        data_content_phash: "phash-gradient:AAAAHAAAXAAAWAAAGAAAmAEA4AEAmgEAkCUApAIAqGsAcXkBgp4ABEcABEcAwL4AMXkBCGcBYHgBAI8ACGMAAA8AYHgBGGcB"
        text_content_hash: "sha256:700c95814d32032eb6d08d7bc7a5f0de92af53400b422a0c8d230d4917ad741f"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: visualize_stroke_6_artifact_json
        data_content_phash: "phash-gradient:AAAAHAAAXAAAWAAAGAAAmAEA4AEAmgEAkCUApAIAqGsAcXkBgp4ABEcABEcAwL4AMXkBCGcBYHgBAI8ACGMAAA8AYHgBGGcB"
        text_content_hash: "sha256:700c95814d32032eb6d08d7bc7a5f0de92af53400b422a0c8d230d4917ad741f"
        "###);

        let done = test_point_iter.next();
        if done.is_some() {
            panic!("test_point_iter is not empty: {}", done.unwrap().name);
        }

        Ok(())
    }
}
