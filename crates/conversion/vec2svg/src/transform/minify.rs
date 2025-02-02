use std::sync::Arc;

use reflexo::TakeAs;

use crate::backend::{SvgText, SvgTextNode};

pub fn minify_one(text: &mut SvgText) -> bool {
    let content = match text {
        SvgText::Plain(_) => return false,
        SvgText::Content(content) => content,
    };

    let content = Arc::make_mut(content);

    if content.content.len() == 1
        && content.attributes.len() == 1
        && content.attributes[0].0 == "transform"
        && matches!(content.content[0], SvgText::Content(_))
    {
        // let transform = content.attributes[0].1.as_str();

        // eprintln!("minify_one fold transform: {:#?}", transform);
        // eprintln!("minify_one fold transform after: {:#?}", transform);

        let sub_content = match &mut content.content[0] {
            SvgText::Plain(_) => unreachable!(),
            SvgText::Content(content) => content.clone(),
        };

        content.content.clear();

        let sub_content = TakeAs::<SvgTextNode>::take(sub_content);

        content.content = sub_content.content;

        for (key, value) in sub_content.attributes {
            if key == "transform" {
                content.attributes[0].1 = format!("{}, {}", content.attributes[0].1, value);
                continue;
            }

            content.attributes.push((key, value));
        }

        *text = SvgText::Content(Arc::new(content.clone()));
        minify_one(text);
        return true;
    }

    let mut optimized = false;

    for text in content.content.iter_mut() {
        let sub = minify_one(text);
        if sub {
            optimized = true;
        }
    }

    if optimized {
        *text = SvgText::Content(Arc::new(content.clone()));
    }

    optimized
}

/// Do semantic-aware minification of SVG.
pub fn minify(mut svg: Vec<SvgText>) -> Vec<SvgText> {
    // eprintln!("minify_svg: {:#?}", svg);

    for text in svg.iter_mut() {
        minify_one(text);
    }

    // eprintln!("minify_svg after: {:#?}", svg);
    svg
}
