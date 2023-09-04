
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page


// Testing show rules with figures with a simple theorem display
#show figure.where(kind: "theorem"): it => {
  let name = none
  if not it.caption == none {
    name = [ #emph(it.caption)]
  } else {
    name = []
  }

  let title = none
  if not it.numbering == none {
    title = it.supplement
    if not it.numbering == none {
      title += " " +  it.counter.display(it.numbering)
    }
  }
  title = strong(title)
  pad(
    top: 0em, bottom: 0em,
    block(
      fill: green.lighten(90%),
      stroke: 1pt + green,
      inset: 10pt,
      width: 100%,
      radius: 5pt,
      breakable: false,
      [#title#name#h(0.1em):#h(0.2em)#it.body#v(0.5em)]
    )
  )
}

#set page(width: 150pt)
#figure(
  $a^2 + b^2 = c^2$,
  supplement: "Theorem",
  kind: "theorem",
  caption: "Pythagoras' theorem.",
  numbering: "1",
) <fig-formula>

#figure(
  $a^2 + b^2 = c^2$,
  supplement: "Theorem",
  kind: "theorem",
  caption: "Another Pythagoras' theorem.",
  numbering: none,
) <fig-formula>

#figure(
  ```rust
  fn main() {
    println!("Hello!");
  }
  ```,
  caption: [Hello world in _rust_],
)
