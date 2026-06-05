
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 60pt)
#context {
  place(right + bottom, rect())
  test(here().position(), (page: 1, x: 10pt, y: 10pt))
}