
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Tests whether the script metrics are used properly by synthesizing
// superscripts and subscripts for all bundled fonts.

#set super(typographic: false)
#set sub(typographic: false)

#let test(font, weights, styles) = {
  for weight in weights {
    for style in styles {
      text(font: font, weight: weight, style: style)[Xx#super[Xx]#sub[Xx]]
      linebreak()
    }
  }
}

#test("DejaVu Sans Mono", ("regular", "bold"), ("normal", "oblique"))
#test("Libertinus Serif", ("regular", "semibold", "bold"), ("normal", "italic"))
#test("New Computer Modern", ("regular", "bold"), ("normal", "italic"))
#test("New Computer Modern Math", (400, 450, "bold"), ("normal",))