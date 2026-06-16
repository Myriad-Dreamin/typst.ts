// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// with unknown region configured
#set text(font: "Noto Serif CJK TC", lang: "zh", region: "XX")
#outline()