
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Markers should only appear on the first page of each item, and further pages
// should be indented.
#set page(width: auto, height: 4em)

- Abc
  def

  ghi
  jkl

  mno
  pqr
- Other
  other

  other
  other