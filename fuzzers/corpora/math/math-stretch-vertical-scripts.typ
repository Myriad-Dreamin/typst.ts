
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test vertical stretch interactions with script attachments.
#let big = $stretch(|, size: #4em)$
$ big_0^1 stretch(|, size: #1.5em)_0^1
  stretch(big, size: #1em)_0^1 |_0^1 $