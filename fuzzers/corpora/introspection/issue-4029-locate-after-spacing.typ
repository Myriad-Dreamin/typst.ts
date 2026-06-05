
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(margin: 10pt)
#show heading: it => v(40pt) + it

= Introduction
#context test(
  locate(heading).position(),
  (page: 1, x: 10pt, y: 50pt),
)