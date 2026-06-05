
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test standard argument overriding.
#{
  let f(style: "normal", weight: "regular") = {
    "(style: " + style + ", weight: " + weight + ")"
  }

  let myf(..args) = f(weight: "bold", ..args)
  test(myf(), "(style: normal, weight: bold)")
  test(myf(weight: "black"), "(style: normal, weight: black)")
  test(myf(style: "italic"), "(style: italic, weight: bold)")
}