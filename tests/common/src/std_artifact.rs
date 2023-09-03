// This file is generated by `cargo run -p std --bin typst-ts-std-test`
#![cfg_attr(rustfmt, rustfmt_skip)]

pub const STD_TEST_FILES: &[(&str, &str)] = &[
    ("bugs", "args-sink_00"),
    ("bugs", "args-underscore_00"),
    ("bugs", "bidi-tofus_00"),
    ("bugs", "cite-locate_00"),
    ("bugs", "clamp-panic_00"),
    ("bugs", "columns-1_00"),
    ("bugs", "flow-1_00"),
    ("bugs", "flow-2_00"),
    ("bugs", "flow-3_00"),
    ("bugs", "flow-4_00"),
    ("bugs", "footnote-keep-multiple_00"),
    ("bugs", "footnote-list_00"),
    ("bugs", "grid-1_00"),
    ("bugs", "grid-1_01"),
    ("bugs", "grid-2_00"),
    ("bugs", "grid-2_01"),
    ("bugs", "grid-3_00"),
    ("bugs", "hide-meta_00"),
    ("bugs", "hide-meta_01"),
    ("bugs", "line-align_00"),
    ("bugs", "math-realize_00"),
    ("bugs", "math-realize_01"),
    ("bugs", "math-realize_02"),
    ("bugs", "parameter-pattern_00"),
    ("bugs", "place-base_00"),
    ("bugs", "place-pagebreak_00"),
    ("bugs", "smartquotes-in-outline_00"),
    ("bugs", "smartquotes-on-newline_00"),
    ("bugs", "square-base_00"),
    ("bugs", "table-lines_00"),
    ("bugs", "table-row-missing_00"),
    ("layout", "align_00"),
    ("layout", "align_01"),
    ("layout", "align_02"),
    ("layout", "align_03"),
    ("layout", "align_04"),
    ("layout", "align_05"),
    ("layout", "block-sizing_00"),
    ("layout", "block-sizing_01"),
    ("layout", "block-spacing_00"),
    ("layout", "clip_00"),
    ("layout", "clip_01"),
    ("layout", "clip_02"),
    ("layout", "clip_03"),
    ("layout", "columns_00"),
    ("layout", "columns_01"),
    ("layout", "columns_02"),
    ("layout", "columns_03"),
    ("layout", "columns_04"),
    ("layout", "columns_05"),
    ("layout", "columns_06"),
    ("layout", "columns_07"),
    ("layout", "columns_08"),
    ("layout", "columns_09"),
    ("layout", "columns_10"),
    ("layout", "container-fill_00"),
    ("layout", "container_00"),
    ("layout", "container_01"),
    ("layout", "container_02"),
    ("layout", "container_03"),
    ("layout", "container_04"),
    ("layout", "enum-align_00"),
    ("layout", "enum-align_01"),
    ("layout", "enum-align_02"),
    ("layout", "enum-align_03"),
    ("layout", "enum-numbering_00"),
    ("layout", "enum-numbering_01"),
    ("layout", "enum-numbering_02"),
    ("layout", "enum-numbering_03"),
    ("layout", "enum-numbering_04"),
    ("layout", "enum-numbering_05"),
    ("layout", "enum-numbering_06"),
    ("layout", "enum_00"),
    ("layout", "enum_01"),
    ("layout", "enum_02"),
    ("layout", "enum_03"),
    ("layout", "enum_04"),
    ("layout", "enum_05"),
    ("layout", "enum_06"),
    ("layout", "flow-orphan_00"),
    ("layout", "flow-orphan_01"),
    ("layout", "grid-1_00"),
    ("layout", "grid-1_01"),
    ("layout", "grid-1_02"),
    ("layout", "grid-2_00"),
    ("layout", "grid-3_00"),
    ("layout", "grid-3_01"),
    ("layout", "grid-3_02"),
    ("layout", "grid-3_03"),
    ("layout", "grid-4_00"),
    ("layout", "grid-4_01"),
    ("layout", "grid-4_02"),
    ("layout", "grid-5_00"),
    ("layout", "grid-5_01"),
    ("layout", "grid-auto-shrink_00"),
    ("layout", "grid-rtl_00"),
    ("layout", "grid-rtl_01"),
    ("layout", "hide_00"),
    ("layout", "list-attach_00"),
    ("layout", "list-attach_01"),
    ("layout", "list-attach_02"),
    ("layout", "list-attach_03"),
    ("layout", "list-attach_04"),
    ("layout", "list-attach_05"),
    ("layout", "list-marker_00"),
    ("layout", "list-marker_01"),
    ("layout", "list-marker_02"),
    ("layout", "list-marker_03"),
    ("layout", "list-marker_04"),
    ("layout", "list_00"),
    ("layout", "list_01"),
    ("layout", "list_02"),
    ("layout", "list_03"),
    ("layout", "list_04"),
    ("layout", "list_05"),
    ("layout", "list_06"),
    ("layout", "list_07"),
    ("layout", "list_08"),
    ("layout", "pad_00"),
    ("layout", "pad_01"),
    ("layout", "pad_02"),
    ("layout", "pad_03"),
    ("layout", "page-binding_00"),
    ("layout", "page-binding_01"),
    ("layout", "page-binding_02"),
    ("layout", "page-binding_03"),
    ("layout", "page-binding_04"),
    ("layout", "page-binding_05"),
    ("layout", "page-margin_00"),
    ("layout", "page-margin_01"),
    ("layout", "page-marginals_00"),
    ("layout", "page-style_00"),
    ("layout", "page-style_01"),
    ("layout", "page-style_02"),
    ("layout", "page-style_03"),
    ("layout", "page_00"),
    ("layout", "page_01"),
    ("layout", "page_02"),
    ("layout", "page_03"),
    ("layout", "page_04"),
    ("layout", "page_05"),
    ("layout", "pagebreak-parity_00"),
    ("layout", "pagebreak-parity_01"),
    ("layout", "pagebreak_00"),
    ("layout", "pagebreak_01"),
    ("layout", "pagebreak_02"),
    ("layout", "pagebreak_03"),
    ("layout", "pagebreak_04"),
    ("layout", "par-bidi_00"),
    ("layout", "par-bidi_01"),
    ("layout", "par-bidi_02"),
    ("layout", "par-bidi_03"),
    ("layout", "par-bidi_04"),
    ("layout", "par-bidi_05"),
    ("layout", "par-bidi_06"),
    ("layout", "par-bidi_07"),
    ("layout", "par-bidi_08"),
    ("layout", "par-indent_00"),
    ("layout", "par-indent_01"),
    ("layout", "par-indent_02"),
    ("layout", "par-indent_03"),
    ("layout", "par-indent_04"),
    ("layout", "par-justify-cjk_00"),
    ("layout", "par-justify-cjk_01"),
    ("layout", "par-justify-cjk_02"),
    ("layout", "par-justify-cjk_03"),
    ("layout", "par-justify_00"),
    ("layout", "par-justify_01"),
    ("layout", "par-justify_02"),
    ("layout", "par-justify_03"),
    ("layout", "par-justify_04"),
    ("layout", "par-knuth_00"),
    ("layout", "par-simple_00"),
    ("layout", "par_00"),
    ("layout", "par_01"),
    ("layout", "par_02"),
    ("layout", "par_03"),
    ("layout", "place-background_00"),
    ("layout", "place-float-auto_00"),
    ("layout", "place-float-auto_01"),
    ("layout", "place-float-auto_02"),
    ("layout", "place-float-figure_00"),
    ("layout", "place-nested_00"),
    ("layout", "place-nested_01"),
    ("layout", "place-nested_02"),
    ("layout", "place-nested_03"),
    ("layout", "place_00"),
    ("layout", "place_01"),
    ("layout", "repeat_00"),
    ("layout", "repeat_01"),
    ("layout", "repeat_02"),
    ("layout", "repeat_03"),
    ("layout", "repeat_04"),
    ("layout", "repeat_05"),
    ("layout", "spacing_00"),
    ("layout", "spacing_01"),
    ("layout", "spacing_02"),
    ("layout", "spacing_03"),
    ("layout", "stack-1_00"),
    ("layout", "stack-1_01"),
    ("layout", "stack-1_02"),
    ("layout", "stack-1_03"),
    ("layout", "stack-2_00"),
    ("layout", "stack-2_01"),
    ("layout", "table_00"),
    ("layout", "table_01"),
    ("layout", "table_02"),
    ("layout", "table_03"),
    ("layout", "table_04"),
    ("layout", "terms_00"),
    ("layout", "terms_01"),
    ("layout", "terms_02"),
    ("layout", "terms_03"),
    ("layout", "terms_04"),
    ("layout", "terms_05"),
    ("layout", "terms_06"),
    ("layout", "transform_00"),
    ("layout", "transform_01"),
    ("layout", "transform_02"),
    ("layout", "transform_03"),
    ("lint", "markup_00"),
    ("lint", "markup_01"),
    ("lint", "markup_02"),
    ("lint", "markup_03"),
    ("math", "accent_00"),
    ("math", "accent_01"),
    ("math", "accent_02"),
    ("math", "accent_03"),
    ("math", "accent_04"),
    ("math", "accent_05"),
    ("math", "accent_06"),
    ("math", "alignment_00"),
    ("math", "alignment_01"),
    ("math", "alignment_02"),
    ("math", "alignment_03"),
    ("math", "attach-p1_00"),
    ("math", "attach-p1_01"),
    ("math", "attach-p1_02"),
    ("math", "attach-p1_03"),
    ("math", "attach-p1_04"),
    ("math", "attach-p2_00"),
    ("math", "attach-p2_01"),
    ("math", "attach-p2_02"),
    ("math", "attach-p2_03"),
    ("math", "attach-p3_00"),
    ("math", "attach-p3_01"),
    ("math", "attach-p3_02"),
    ("math", "attach-p3_03"),
    ("math", "attach-p3_04"),
    ("math", "attach-p3_05"),
    ("math", "attach-p3_06"),
    ("math", "cancel_00"),
    ("math", "cancel_01"),
    ("math", "cancel_02"),
    ("math", "cancel_03"),
    ("math", "cancel_04"),
    ("math", "cancel_05"),
    ("math", "cases_00"),
    ("math", "class_00"),
    ("math", "class_01"),
    ("math", "class_02"),
    ("math", "class_03"),
    ("math", "content_00"),
    ("math", "content_01"),
    ("math", "content_02"),
    ("math", "content_03"),
    ("math", "content_04"),
    ("math", "content_05"),
    ("math", "delimited_00"),
    ("math", "delimited_01"),
    ("math", "delimited_02"),
    ("math", "delimited_03"),
    ("math", "delimited_04"),
    ("math", "delimited_05"),
    ("math", "delimited_06"),
    ("math", "delimited_07"),
    ("math", "delimited_08"),
    ("math", "font-features_00"),
    ("math", "frac_00"),
    ("math", "frac_01"),
    ("math", "frac_02"),
    ("math", "frac_03"),
    ("math", "frac_04"),
    ("math", "frac_05"),
    ("math", "frac_06"),
    ("math", "frac_07"),
    ("math", "matrix-alignment_00"),
    ("math", "matrix-alignment_01"),
    ("math", "matrix-alignment_02"),
    ("math", "matrix-alignment_03"),
    ("math", "matrix-alignment_04"),
    ("math", "matrix-alignment_05"),
    ("math", "matrix-alignment_06"),
    ("math", "matrix_00"),
    ("math", "matrix_01"),
    ("math", "matrix_02"),
    ("math", "matrix_03"),
    ("math", "matrix_04"),
    ("math", "matrix_05"),
    ("math", "matrix_06"),
    ("math", "multiline_00"),
    ("math", "multiline_01"),
    ("math", "multiline_02"),
    ("math", "multiline_03"),
    ("math", "multiline_04"),
    ("math", "multiline_05"),
    ("math", "multiline_06"),
    ("math", "multiline_07"),
    ("math", "multiline_08"),
    ("math", "numbering_00"),
    ("math", "op_00"),
    ("math", "op_01"),
    ("math", "op_02"),
    ("math", "op_03"),
    ("math", "op_04"),
    ("math", "opticalsize_00"),
    ("math", "opticalsize_01"),
    ("math", "opticalsize_02"),
    ("math", "opticalsize_03"),
    ("math", "opticalsize_04"),
    ("math", "opticalsize_05"),
    ("math", "opticalsize_06"),
    ("math", "root_00"),
    ("math", "root_01"),
    ("math", "root_02"),
    ("math", "root_03"),
    ("math", "root_04"),
    ("math", "root_05"),
    ("math", "spacing_00"),
    ("math", "spacing_01"),
    ("math", "spacing_02"),
    ("math", "spacing_03"),
    ("math", "spacing_04"),
    ("math", "style_00"),
    ("math", "style_01"),
    ("math", "style_02"),
    ("math", "style_03"),
    ("math", "style_04"),
    ("math", "style_05"),
    ("math", "style_06"),
    ("math", "style_07"),
    ("math", "syntax_00"),
    ("math", "syntax_01"),
    ("math", "syntax_02"),
    ("math", "syntax_03"),
    ("math", "unbalanced_00"),
    ("math", "underover_00"),
    ("math", "underover_01"),
    ("math", "underover_02"),
    ("math", "vec_00"),
    ("math", "vec_01"),
    ("math", "vec_02"),
    ("meta", "bibliography-ordering_00"),
    ("meta", "bibliography_00"),
    ("meta", "bibliography_01"),
    ("meta", "bibliography_02"),
    ("meta", "bibliography_03"),
    ("meta", "bibliography_04"),
    ("meta", "cite-footnote_00"),
    ("meta", "counter-page_00"),
    ("meta", "counter_00"),
    ("meta", "counter_01"),
    ("meta", "counter_02"),
    ("meta", "counter_03"),
    ("meta", "document_00"),
    ("meta", "document_01"),
    ("meta", "document_02"),
    ("meta", "document_03"),
    ("meta", "document_04"),
    ("meta", "document_05"),
    ("meta", "document_06"),
    ("meta", "document_07"),
    ("meta", "figure_00"),
    ("meta", "figure_01"),
    ("meta", "figure_02"),
    ("meta", "figure_03"),
    ("meta", "footnote-break_00"),
    ("meta", "footnote-columns_00"),
    ("meta", "footnote-container_00"),
    ("meta", "footnote-container_01"),
    ("meta", "footnote-invariant_00"),
    ("meta", "footnote-refs_00"),
    ("meta", "footnote-refs_01"),
    ("meta", "footnote-refs_02"),
    ("meta", "footnote-refs_03"),
    ("meta", "footnote-refs_04"),
    ("meta", "footnote-refs_05"),
    ("meta", "footnote-table_00"),
    ("meta", "footnote_00"),
    ("meta", "footnote_01"),
    ("meta", "footnote_02"),
    ("meta", "footnote_03"),
    ("meta", "footnote_04"),
    ("meta", "heading_00"),
    ("meta", "heading_01"),
    ("meta", "heading_02"),
    ("meta", "heading_03"),
    ("meta", "heading_04"),
    ("meta", "link_00"),
    ("meta", "link_01"),
    ("meta", "link_02"),
    ("meta", "link_03"),
    ("meta", "link_04"),
    ("meta", "link_05"),
    ("meta", "link_06"),
    ("meta", "link_07"),
    ("meta", "link_08"),
    ("meta", "link_09"),
    ("meta", "link_10"),
    ("meta", "link_11"),
    ("meta", "numbering_00"),
    ("meta", "numbering_01"),
    ("meta", "numbering_02"),
    ("meta", "numbering_03"),
    ("meta", "numbering_04"),
    ("meta", "numbering_05"),
    ("meta", "numbering_06"),
    ("meta", "outline-entry_00"),
    ("meta", "outline-entry_01"),
    ("meta", "outline-entry_02"),
    ("meta", "outline-indent_00"),
    ("meta", "outline-indent_01"),
    ("meta", "outline-indent_02"),
    ("meta", "outline_00"),
    ("meta", "query-before-after_00"),
    ("meta", "query-before-after_01"),
    ("meta", "query-figure_00"),
    ("meta", "query-header_00"),
    ("meta", "ref_00"),
    ("meta", "ref_01"),
    ("meta", "ref_02"),
    ("meta", "ref_03"),
    ("meta", "state_00"),
    ("meta", "state_01"),
    ("meta", "state_02"),
    ("meta", "state_03"),
    ("text", "baseline_00"),
    ("text", "baseline_01"),
    ("text", "case_00"),
    ("text", "case_01"),
    ("text", "chinese_00"),
    ("text", "copy-paste_00"),
    ("text", "deco_00"),
    ("text", "deco_01"),
    ("text", "deco_02"),
    ("text", "edge_00"),
    ("text", "edge_01"),
    ("text", "edge_02"),
    ("text", "edge_03"),
    ("text", "em_00"),
    ("text", "em_01"),
    ("text", "emoji_00"),
    ("text", "emoji_01"),
    ("text", "emphasis_00"),
    ("text", "emphasis_01"),
    ("text", "emphasis_02"),
    ("text", "emphasis_03"),
    ("text", "emphasis_04"),
    ("text", "emphasis_05"),
    ("text", "escape_00"),
    ("text", "escape_01"),
    ("text", "escape_02"),
    ("text", "fallback_00"),
    ("text", "features_00"),
    ("text", "features_01"),
    ("text", "features_02"),
    ("text", "features_03"),
    ("text", "features_04"),
    ("text", "features_05"),
    ("text", "features_06"),
    ("text", "features_07"),
    ("text", "features_08"),
    ("text", "features_09"),
    ("text", "features_10"),
    ("text", "features_11"),
    ("text", "features_12"),
    ("text", "font_00"),
    ("text", "font_01"),
    ("text", "font_02"),
    ("text", "font_03"),
    ("text", "font_04"),
    ("text", "font_05"),
    ("text", "hyphenate_00"),
    ("text", "hyphenate_01"),
    ("text", "hyphenate_02"),
    ("text", "hyphenate_03"),
    ("text", "hyphenate_04"),
    ("text", "lang-with-region_00"),
    ("text", "lang-with-region_01"),
    ("text", "lang-with-region_02"),
    ("text", "lang_00"),
    ("text", "lang_01"),
    ("text", "lang_02"),
    ("text", "lang_03"),
    ("text", "lang_04"),
    ("text", "lang_05"),
    ("text", "lang_06"),
    ("text", "lang_07"),
    ("text", "lang_08"),
    ("text", "linebreak-obj_00"),
    ("text", "linebreak-obj_01"),
    ("text", "linebreak_00"),
    ("text", "linebreak_01"),
    ("text", "linebreak_02"),
    ("text", "linebreak_03"),
    ("text", "linebreak_04"),
    ("text", "linebreak_05"),
    ("text", "linebreak_06"),
    ("text", "linebreak_07"),
    ("text", "linebreak_08"),
    ("text", "linebreak_09"),
    ("text", "lorem_00"),
    ("text", "lorem_01"),
    ("text", "lorem_02"),
    ("text", "microtype_00"),
    ("text", "microtype_01"),
    ("text", "quotes_00"),
    ("text", "quotes_01"),
    ("text", "quotes_02"),
    ("text", "quotes_03"),
    ("text", "quotes_04"),
    ("text", "quotes_05"),
    ("text", "raw-align_00"),
    ("text", "raw-align_01"),
    ("text", "raw-align_02"),
    ("text", "raw-code_00"),
    ("text", "raw-syntaxes_00"),
    ("text", "raw-theme_00"),
    ("text", "raw_00"),
    ("text", "raw_01"),
    ("text", "raw_02"),
    ("text", "raw_03"),
    ("text", "raw_04"),
    ("text", "raw_05"),
    ("text", "raw_06"),
    ("text", "raw_07"),
    ("text", "raw_08"),
    ("text", "raw_09"),
    ("text", "raw_10"),
    ("text", "raw_11"),
    ("text", "shaping_00"),
    ("text", "shaping_01"),
    ("text", "shaping_02"),
    ("text", "shaping_03"),
    ("text", "shift_00"),
    ("text", "shift_01"),
    ("text", "shift_02"),
    ("text", "space_00"),
    ("text", "space_01"),
    ("text", "space_02"),
    ("text", "space_03"),
    ("text", "space_04"),
    ("text", "space_05"),
    ("text", "space_06"),
    ("text", "symbol_00"),
    ("text", "symbol_01"),
    ("text", "tracking-spacing_00"),
    ("text", "tracking-spacing_01"),
    ("text", "tracking-spacing_02"),
    ("text", "tracking-spacing_03"),
    ("text", "tracking-spacing_04"),
    ("text", "tracking-spacing_05"),
    ("visualize", "image_00"),
    ("visualize", "image_01"),
    ("visualize", "image_02"),
    ("visualize", "image_03"),
    ("visualize", "image_04"),
    ("visualize", "image_05"),
    ("visualize", "image_06"),
    ("visualize", "image_07"),
    ("visualize", "image_08"),
    ("visualize", "image_09"),
    ("visualize", "image_10"),
    ("visualize", "image_11"),
    ("visualize", "image_12"),
    ("visualize", "image_13"),
    ("visualize", "line_00"),
    ("visualize", "line_01"),
    ("visualize", "line_02"),
    ("visualize", "line_03"),
    ("visualize", "path_00"),
    ("visualize", "path_01"),
    ("visualize", "path_02"),
    ("visualize", "path_03"),
    ("visualize", "polygon_00"),
    ("visualize", "polygon_01"),
    ("visualize", "shape-aspect_00"),
    ("visualize", "shape-aspect_01"),
    ("visualize", "shape-aspect_02"),
    ("visualize", "shape-aspect_03"),
    ("visualize", "shape-aspect_04"),
    ("visualize", "shape-aspect_05"),
    ("visualize", "shape-aspect_06"),
    ("visualize", "shape-circle_00"),
    ("visualize", "shape-circle_01"),
    ("visualize", "shape-circle_02"),
    ("visualize", "shape-circle_03"),
    ("visualize", "shape-circle_04"),
    ("visualize", "shape-ellipse_00"),
    ("visualize", "shape-ellipse_01"),
    ("visualize", "shape-fill-stroke_00"),
    ("visualize", "shape-fill-stroke_01"),
    ("visualize", "shape-fill-stroke_02"),
    ("visualize", "shape-rect_00"),
    ("visualize", "shape-rect_01"),
    ("visualize", "shape-rect_02"),
    ("visualize", "shape-rect_03"),
    ("visualize", "shape-rect_04"),
    ("visualize", "shape-rounded_00"),
    ("visualize", "shape-square_00"),
    ("visualize", "shape-square_01"),
    ("visualize", "shape-square_02"),
    ("visualize", "shape-square_03"),
    ("visualize", "shape-square_04"),
    ("visualize", "shape-square_05"),
    ("visualize", "stroke_00"),
    ("visualize", "stroke_01"),
    ("visualize", "stroke_02"),
    ("visualize", "stroke_03"),
    ("visualize", "stroke_04"),
    ("visualize", "stroke_05"),
    ("visualize", "stroke_06"),
    ("visualize", "stroke_07"),
    ("visualize", "svg-text_00")
];
