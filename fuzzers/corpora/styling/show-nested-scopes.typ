
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that scoping works as expected.
#{
  let world = [ World ]
  show "W": strong
  world
  {
    set text(blue)
    show: it => {
      show "o": "Ø"
      it
    }
    world
  }
  world
}