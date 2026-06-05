
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// These are both red because in the expanded form, `set text(red)` ends up
// closer to the content than `set text(blue)`.
#show strong: it => { set text(red); it }
Hello *World*

#show strong: it => { set text(blue); it }
Hello *World*