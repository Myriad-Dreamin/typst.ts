
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test({
  let foo(x) = {
    if x < 0 { "negative" }
    // comment
    else { "non-negative" }
  }

  foo(1)
}, "non-negative")