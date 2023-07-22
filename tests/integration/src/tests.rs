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
        origin_pdf_hash: "sha256:b6437eb720e2dc8dfd60f71227fa8aedee251c5b7c145ae50aea39b6e9e45507"
        artifact_pdf_hash: "sha256:b6437eb720e2dc8dfd60f71227fa8aedee251c5b7c145ae50aea39b6e9e45507"
        "###);
        check_bundle_facts!("layout", "clip_2", @r###"
        ---
        name: layout/clip_2.typ
        origin_pdf_hash: "sha256:78e4aaa4640f27533f8df1eba3b96c657f569fd69f7d1cc5cad432bb2c70438d"
        artifact_pdf_hash: "sha256:78e4aaa4640f27533f8df1eba3b96c657f569fd69f7d1cc5cad432bb2c70438d"
        "###);
        check_bundle_facts!("layout", "clip_3", @r###"
        ---
        name: layout/clip_3.typ
        origin_pdf_hash: "sha256:81c13609847913bc0cf8961d7a60f64e46300e1eba2b70802cfbd3df6722d0f8"
        artifact_pdf_hash: "sha256:81c13609847913bc0cf8961d7a60f64e46300e1eba2b70802cfbd3df6722d0f8"
        "###);
        check_bundle_facts!("layout", "clip_4", @r###"
        ---
        name: layout/clip_4.typ
        origin_pdf_hash: "sha256:85d345941d82d57d7a9f86c7509ca6b3fffdd44bc1adf13da63c5545b9fc7908"
        artifact_pdf_hash: "sha256:85d345941d82d57d7a9f86c7509ca6b3fffdd44bc1adf13da63c5545b9fc7908"
        "###);

        check_bundle_facts!("layout", "list_marker_1", @r###"
        ---
        name: layout/list_marker_1.typ
        origin_pdf_hash: "sha256:99fbfa5c4edbe141af9bfa5ccb16a4e4f188c92297d8a5cb1928b5fd65973345"
        artifact_pdf_hash: "sha256:99fbfa5c4edbe141af9bfa5ccb16a4e4f188c92297d8a5cb1928b5fd65973345"
        "###);
        check_bundle_facts!("layout", "list_marker_2", @r###"
        ---
        name: layout/list_marker_2.typ
        origin_pdf_hash: "sha256:e7588ed6f34fa428c9f9c913755aa211809f4e5644f88b44d7fa65c53a3b1d5f"
        artifact_pdf_hash: "sha256:e7588ed6f34fa428c9f9c913755aa211809f4e5644f88b44d7fa65c53a3b1d5f"
        "###);
        check_bundle_facts!("layout", "list_marker_3", @r###"
        ---
        name: layout/list_marker_3.typ
        origin_pdf_hash: "sha256:7d56e142562e33cab2ac2527520ef56a7dc08300ad984278b4062eaaf0bd8ce2"
        artifact_pdf_hash: "sha256:7d56e142562e33cab2ac2527520ef56a7dc08300ad984278b4062eaaf0bd8ce2"
        "###);
        check_bundle_facts!("layout", "list_marker_4", @r###"
        ---
        name: layout/list_marker_4.typ
        origin_pdf_hash: "sha256:4d776c3a50e63277219475df469fd5ca11d0d5c6bcfe62dc7a4c53463b690a84"
        artifact_pdf_hash: "sha256:4d776c3a50e63277219475df469fd5ca11d0d5c6bcfe62dc7a4c53463b690a84"
        "###);

        check_bundle_facts!("layout", "transform_1", @r###"
        ---
        name: layout/transform_1.typ
        origin_pdf_hash: "sha256:cb1d6880801b04f42b4aa702f49f4a84b6fdd17a56ec42eff65a45c643c245ef"
        artifact_pdf_hash: "sha256:cb1d6880801b04f42b4aa702f49f4a84b6fdd17a56ec42eff65a45c643c245ef"
        "###);
        check_bundle_facts!("layout", "transform_2", @r###"
        ---
        name: layout/transform_2.typ
        origin_pdf_hash: "sha256:3627a123dc74c917ef1f3ca216909997b27a9394bbf3b42b1c4210f772f75108"
        artifact_pdf_hash: "sha256:3627a123dc74c917ef1f3ca216909997b27a9394bbf3b42b1c4210f772f75108"
        "###);
        check_bundle_facts!("layout", "transform_3", @r###"
        ---
        name: layout/transform_3.typ
        origin_pdf_hash: "sha256:4b34f08261293150aa0bc03df3db4c956b1b7207e9f5a19471f746841d8512b4"
        artifact_pdf_hash: "sha256:4b34f08261293150aa0bc03df3db4c956b1b7207e9f5a19471f746841d8512b4"
        "###);
        check_bundle_facts!("layout", "transform_4", @r###"
        ---
        name: layout/transform_4.typ
        origin_pdf_hash: "sha256:f8cb255abd54d67e2795d55f8f96f204b65bffb4c04d2f70af77c54db9daa9b1"
        artifact_pdf_hash: "sha256:f8cb255abd54d67e2795d55f8f96f204b65bffb4c04d2f70af77c54db9daa9b1"
        "###);

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
        check_bundle_facts!("visualize", "shape_circle_1", @r###"
        ---
        name: visualize/shape_circle_1.typ
        origin_pdf_hash: "sha256:2af37fc60a4cb4abc8374b157c171cf3478f0d1bb354c84ad7b438662dfa9736"
        artifact_pdf_hash: "sha256:2af37fc60a4cb4abc8374b157c171cf3478f0d1bb354c84ad7b438662dfa9736"
        "###);
        check_bundle_facts!("visualize", "shape_circle_2", @r###"
        ---
        name: visualize/shape_circle_2.typ
        origin_pdf_hash: "sha256:b8d22281e35b43118d0e5331d690bd41b49d98758b6389884e6959456b3cecb3"
        artifact_pdf_hash: "sha256:b8d22281e35b43118d0e5331d690bd41b49d98758b6389884e6959456b3cecb3"
        "###);
        check_bundle_facts!("visualize", "shape_circle_3", @r###"
        ---
        name: visualize/shape_circle_3.typ
        origin_pdf_hash: "sha256:637f5265a3b25f29cca0e1053640307e01dc407087a9d070d21005642d53e6e3"
        artifact_pdf_hash: "sha256:637f5265a3b25f29cca0e1053640307e01dc407087a9d070d21005642d53e6e3"
        "###);
        check_bundle_facts!("visualize", "shape_circle_4", @r###"
        ---
        name: visualize/shape_circle_4.typ
        origin_pdf_hash: "sha256:d9f1073eb598a62f75f669bcdf08010efdba3980fa74c7f065668b0377a2354e"
        artifact_pdf_hash: "sha256:d9f1073eb598a62f75f669bcdf08010efdba3980fa74c7f065668b0377a2354e"
        "###);
        // todo: typst cannot pass visualize/stroke_4 test.
        check_bundle_facts!("visualize", "stroke_1", @r###"
        ---
        name: visualize/stroke_1.typ
        origin_pdf_hash: "sha256:64734195769470c8fbe95f0d501a7b73e61f25d63513718691bad2ab4e3d5de8"
        artifact_pdf_hash: "sha256:64734195769470c8fbe95f0d501a7b73e61f25d63513718691bad2ab4e3d5de8"
        "###);
        check_bundle_facts!("visualize", "stroke_2", @r###"
        ---
        name: visualize/stroke_2.typ
        origin_pdf_hash: "sha256:32c7db206ace75f0b3cb7c58392e0477f3959a83f759fd60edc073e841f20abf"
        artifact_pdf_hash: "sha256:32c7db206ace75f0b3cb7c58392e0477f3959a83f759fd60edc073e841f20abf"
        "###);
        check_bundle_facts!("visualize", "stroke_3", @r###"
        ---
        name: visualize/stroke_3.typ
        origin_pdf_hash: "sha256:65103be4a373417258b964aadb7bc3ade60bebd73181b30e780954dd69cd0aff"
        artifact_pdf_hash: "sha256:65103be4a373417258b964aadb7bc3ade60bebd73181b30e780954dd69cd0aff"
        "###);
        check_bundle_facts!("visualize", "stroke_4", @r###"
        ---
        name: visualize/stroke_4.typ
        origin_pdf_hash: "sha256:a705e05dd32ff91407373e49e5f939c592a37f0c8b61903ad1ea0c7395a0aa63"
        artifact_pdf_hash: "sha256:a705e05dd32ff91407373e49e5f939c592a37f0c8b61903ad1ea0c7395a0aa63"
        "###);
        check_bundle_facts!("visualize", "stroke_5", @r###"
        ---
        name: visualize/stroke_5.typ
        origin_pdf_hash: "sha256:2df82172746635e220bb8ad7173070e479c3bce78968933d3df57865e81f0bca"
        artifact_pdf_hash: "sha256:2df82172746635e220bb8ad7173070e479c3bce78968933d3df57865e81f0bca"
        "###);
        check_bundle_facts!("visualize", "stroke_6", @r###"
        ---
        name: visualize/stroke_6.typ
        origin_pdf_hash: "sha256:56baa4dd6324590e28be5f0a9cd632f6cc3a3396e6de2fae01b3c7a8aa6cfd19"
        artifact_pdf_hash: "sha256:56baa4dd6324590e28be5f0a9cd632f6cc3a3396e6de2fae01b3c7a8aa6cfd19"
        "###);

        check_bundle_facts!("text", "chinese", @r###"
        ---
        name: text/chinese.typ
        origin_pdf_hash: "sha256:de94be2f518feaa75dc0647d340f38272b01151c9bc6223e82f6c2190f697b43"
        artifact_pdf_hash: "sha256:de94be2f518feaa75dc0647d340f38272b01151c9bc6223e82f6c2190f697b43"
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
        // todo: typst cannot pass visualize/stroke_6 test.

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
        data_content_phash: "phash-gradient:AAAAgMwAAMQAmAYA2MYAAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:98c5b7172c1fb068bd716678b1eb9dd73941d9ae5a44fecb2550a970c9407777"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: math_main_artifact_json
        data_content_phash: "phash-gradient:AAAAgMwAAMQAmAYA2MYAAMgAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAABAA"
        text_content_hash: "sha256:98c5b7172c1fb068bd716678b1eb9dd73941d9ae5a44fecb2550a970c9407777"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: math_undergradmath_artifact_ir
        data_content_phash: "phash-gradient:YGMAQMtNZIzN7CXxZujtZKoW5AT1cGbRLG9mmI1ilIQ2MINiMMNmOEdmMMdkOMPmmGbmmEZkmYbW2GzONInhUAxjSY43KAYA"
        text_content_hash: "sha256:3968674673649bbdbac381e31bc464e6f311821332abf26e511ea31e357e6d31"
        "###);
        check_canvas_render_test_point!(@r###"
        ---
        name: math_undergradmath_artifact_json
        data_content_phash: "phash-gradient:YGMAQMtNZIzN7CXxZujtZKoW5AT1cGbRLG9mmI1ilIQ2MINiMMNmOEdmMMdkOMPmmGbmmEZkmYbW2GzONInhUAxjSY43KAYA"
        text_content_hash: "sha256:3968674673649bbdbac381e31bc464e6f311821332abf26e511ea31e357e6d31"
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
