
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether cjk-latin-spacing would be stretched evenly when justified.
#set par(justify: true)
あaあ#linebreak(justify: true)
ああaa aaああ#linebreak(justify: true)