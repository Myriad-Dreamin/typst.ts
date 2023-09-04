
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test spacing with comments.
A/**/B/**/C \
A /**/ B/**/C \
A /**/B/**/ C
