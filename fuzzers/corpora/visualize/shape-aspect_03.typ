
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test square that is limited by region size.
#set page(width: 20pt, height: 10pt, margin: 0pt)
#stack(dir: ltr, square(fill: forest), square(fill: conifer))
