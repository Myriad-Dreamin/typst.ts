
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show " ": it => {
  test(it.func(), text)
  test(it.text, " ")
  [-]
}
// We split up the text run into three separate elements to see what kind of
// element we get in the match (space vs text). We want text so that a `.text`
// field is available.
A#[ ]B