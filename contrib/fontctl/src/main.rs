use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "typst-ts-fontctl", version = "0.1.0")]
pub struct Opts {
    /// Use font assets in branch of typst
    #[clap(long, default_value = "")]
    pub using_branch: String,
}

const FONT_LISTS: &[(&'static str, &'static str)] = &[
    (
        "DejaVuSansMono-Bold.ttf",
        "bce60f1b4421acd9ea51ba6623d7024ecbe6817a953e3654df62a5e6bdf8f769",
    ),
    (
        "DejaVuSansMono-BoldOblique.ttf",
        "91713a71d550bba22c2a6b2bb2a9ad8f9a159e12e4e9f0a5b2677998ba21213e",
    ),
    (
        "DejaVuSansMono-Oblique.ttf",
        "742097840c541870e8d6dc5c9b37bb1ceeea6c0dedd1d475faf903ef9df734b0",
    ),
    (
        "DejaVuSansMono.ttf",
        "b4a6c3e4faab8773f4ff761d56451646409f29abedd68f05d38c2df667d3c582",
    ),
    (
        "FiraMath-Regular.otf",
        "2028cbd3dd4d8c0cf1608520eb4759956a83a67931d7b6d8e7c313520186e35b",
    ),
    (
        "IBMPlexSerif-Regular.ttf",
        "29f1b4fbda490747553b47780feb1b73832c48581894ebf1caa98959a024fdc9",
    ),
    (
        "InriaSerif-BoldItalic.ttf",
        "d7c140cdb61a4c1edca17abed1d9d57fe5b345d0195d58129c98e61e8cc30f9b",
    ),
    (
        "InriaSerif-Regular.ttf",
        "907c200e38899bad5668d81b4c0fd3e880d7da051142405c3ccd7b1c2f45edb5",
    ),
    (
        "LinLibertine_R.ttf",
        "06e2b67aa9a899e71f78b91da21cd0bba0fb1c5791fd3b58752ea25a4b9b79d7",
    ),
    (
        "LinLibertine_RB.ttf",
        "b044e2233db4f1a792e8422827679f3bc3b573c164597e9d3b03a63f123b4cdf",
    ),
    (
        "LinLibertine_RBI.ttf",
        "830a780c355f8b577c7d34d47ccc45e5a99f2117a9e1ee4a37ee5b71de8eb8da",
    ),
    (
        "LinLibertine_RI.ttf",
        "8006a620ae7487b45c8279cc51b676ec0209af7151fea00d69df44142171e7b6",
    ),
    (
        "NewCM10-Bold.otf",
        "515b2e3cf32e472fac2eff9ae11ef82b43214ddbaf48e11055228866308ec9dd",
    ),
    (
        "NewCM10-BoldItalic.otf",
        "e8c5a5b70c4b144d25cb6efd51713912c66c087d1982a2901617dd8f20ca990d",
    ),
    (
        "NewCM10-Italic.otf",
        "d839cd35577f303e7bc018e85f5f532be84911d032b0026d028cf03b58a8727f",
    ),
    (
        "NewCM10-Regular.otf",
        "b4c94f7d37deb9272481c1838d6e56049913a8c20dcc80c60ce4fd72d03452cd",
    ),
    (
        "NewCMMath-Book.otf",
        "5935eae267430f4c0e06457d4af63c2b1457a402802524094d797151eeee4797",
    ),
    (
        "NewCMMath-Regular.otf",
        "9d71891da05f97df060dc92337f2cd7fc4ca5abbc88bf06363542bd5c54c718a",
    ),
    (
        "NotoColorEmoji.ttf",
        "bf2a8506b80614ba190a34c7b037af1269a7d614fe9f3b613cc15cdeec6f814b",
    ),
    (
        "NotoSansArabic-Regular.ttf",
        "88eb8b26e974763c19df902694bc3c581ea492dcff8fbd66ffa29bc1a9a61801",
    ),
    (
        "NotoSansSymbols2-Regular.ttf",
        "882d142b9a1ef3fd7fa4225dbe95c10fab6664206eb4964c8ff705a4f6d02988",
    ),
    (
        "NotoSerifCJKsc-Regular.otf",
        "efa5e49879a789c1ecad8f07e1813c7f9aa6c4b951fdbb5791d85d67345632d7",
    ),
    (
        "NotoSerifHebrew-Bold.ttf",
        "1a512eff39dcf43d68702e4f6975d7d8af5e5252940237255c80b95cde5a0730",
    ),
    (
        "NotoSerifHebrew-Regular.ttf",
        "92b171e3b48a2d2bc3cce02bd6143958663b2fd4ab0fdb77cfb2865651635398",
    ),
    (
        "PTSans-Regular.ttf",
        "419e240303f11800c2b0d24b19bd361831be1789142586aeca5bf078a1e7733e",
    ),
    (
        "Roboto-Regular.ttf",
        "797e35f7f5d6020a5c6ea13b42ecd668bcfb3bbc4baa0e74773527e5b6cb3174",
    ),
    (
        "TwitterColorEmoji.ttf",
        "11d45f5afc6a6a19f0c99831bc47f879e4157a8675cfaa436529f4ef227ddbcb",
    ),
    (
        "Ubuntu-Regular.ttf",
        "66fea9c00091f25eb8a526548023b6154785876a900af2d8f472922689698163",
    ),
];

fn download_fonts(opts: Opts) {
    let mut repo = typst_ts_fontctl::git_download::repo("https://github.com/typst/typst")
        .branch_name({
            if !opts.using_branch.is_empty() {
                &opts.using_branch
            } else {
                "623db8c0c4cd798d3dacaa5269f9e850fc180934"
            }
        });

    for font in FONT_LISTS {
        println!("add font: {}", font.0);
        repo = repo.add_file(
            format!("assets/fonts/{}", font.0),
            format!("assets/fonts/{}", font.0),
        );
    }

    println!("downloading...");
    repo.exec().unwrap();

    for font in FONT_LISTS {
        std::fs::copy(
            format!("assets/fonts/{}", font.0),
            format!("packages/renderer/dist/fonts/{}", font.0),
        )
        .unwrap();
    }

    println!("done");
}

fn main() {
    let opts = Opts::parse();

    download_fonts(opts);
}
