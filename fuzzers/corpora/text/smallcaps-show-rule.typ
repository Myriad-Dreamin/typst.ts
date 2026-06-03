
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// There is no dedicated smallcaps font in typst-dev-assets, so we just use some
// other font to test this show rule.
#show smallcaps: set text(font: "PT Sans")
#smallcaps[Smallcaps]

#show smallcaps: set text(fill: red)
#smallcaps[Smallcaps]