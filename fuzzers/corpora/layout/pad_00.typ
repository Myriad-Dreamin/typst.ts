
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Use for indentation.
#pad(left: 10pt, [Indented!])

// All sides together.
#set rect(inset: 0pt)
#rect(fill: conifer,
  pad(10pt, right: 20pt,
    rect(width: 20pt, height: 20pt, fill: rgb("eb5278"))
  )
)

Hi #box(pad(left: 10pt)[A]) there
