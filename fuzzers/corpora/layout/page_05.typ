
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Layout without any container should provide the page's dimensions, minus its margins.

#page(width: 100pt, height: 100pt, {
  layout(size => [This page has a width of #size.width and height of #size.height ])
  h(1em)
  place(left, rect(width: 80pt, stroke: blue))
})
