
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test disabling the dotless glyph variants.
$hat(i), hat(i, dotless: #false), accent(j, tilde), accent(j, tilde, dotless: #false)$