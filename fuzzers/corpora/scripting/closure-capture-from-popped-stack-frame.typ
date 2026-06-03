
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Capture environment.
#{
  let mark = "!"
  let greet = {
    let hi = "Hi"
    name => {
        hi + ", " + name + mark
    }
  }

  test(greet("Typst"), "Hi, Typst!")

  // Changing the captured variable after the closure definition has no effect.
  mark = "?"
  test(greet("Typst"), "Hi, Typst!")
}