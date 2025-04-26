
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(
  paper: "a7",
  margin: (y: 1cm, x: 0.5cm),
  header: {
    smallcaps[Typst Academy]
    h(1fr)
    context {
      let loc = here();
      let after = query(selector(heading).after(loc))
      let before = query(selector(heading).before(loc))
      let elem = if before.len() != 0 {
        before.last()
      } else if after.len() != 0 {
        after.first()
      }
      emph(elem.body)
    }
  }
)

#outline()

= Introduction
#lorem(35)

= Background
#lorem(35)

= Approach
#lorem(60)
