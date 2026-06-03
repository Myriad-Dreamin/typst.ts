
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that the heading's tag isn't stuck at the end of the paragraph.
#set page(margin: 10pt)
Par
#show heading: it => pagebreak() + it
= Introduction
#context test(locate(heading).page(), 2)