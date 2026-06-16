// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that math does not produce paragraphs.
#show par: highlight
$ a + "bc" + #[c] + #box[d] + #block[e] $