
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test effect of bottom accent on subscript.
$q_x != accent(q, \u{032C})_x != accent(accent(q, \u{032C}), \u{032C})_x$