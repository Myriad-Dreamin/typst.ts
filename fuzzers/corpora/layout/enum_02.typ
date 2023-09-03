
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test automatic numbering in summed content.
#for i in range(5) {
   [+ #numbering("I", 1 + i)]
}
