
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test bottom accent size.
$accent(sum, \u{0330}), accent(sum, \u{0330}, size: #50%), accent(H, \u{032D}, size: #200%)$