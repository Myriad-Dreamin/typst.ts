
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(
  paper: "a8",
  margin: (y: 1cm, x: 0.5cm),
  header: context {
    smallcaps[Typst Academy]
    h(1fr)
    let after = query(selector(heading).after(here()))
    let before = query(selector(heading).before(here()))
    let elem = if before.len() != 0 {
      before.last()
    } else if after.len() != 0 {
      after.first()
    }
    emph(elem.body)
  }
)

#outline()

= Introduction
#lines(1)

= Background
#lines(2)

= Approach