
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test turning kerning off.
#text(kerning: true)[Tq] \
#text(kerning: false)[Tq]
