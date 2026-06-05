
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test accessing the string itself.
#show "hello": it => it.text.split("").map(upper).join("|")
Oh, hello there!