
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(margin: 10pt)
#show heading: it => pagebreak() + it

= Introduction
#context test(
  locate(heading).position(),
  (page: 2, x: 10pt, y: 10pt),
)