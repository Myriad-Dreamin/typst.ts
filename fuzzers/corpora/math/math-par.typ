
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that math does not produce paragraphs.
#show par: highlight
$ a + "bc" + #[c] + #box[d] + #block[e] $