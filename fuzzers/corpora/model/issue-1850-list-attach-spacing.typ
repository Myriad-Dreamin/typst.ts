
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// List attachment should only work with paragraphs, not other blocks.
#set page(width: auto)
#let part = box.with(stroke: 1pt, inset: 3pt)
#{
  part[
    $ x $
    - A
  ]
  part($ x $ + list[A])
  part($ x $ + list[ A ])
  part[
    $ x $

    - A
  ]
  part($ x $ + parbreak() + list[A])
  part($ x $ + parbreak() + parbreak() + list[A])
}