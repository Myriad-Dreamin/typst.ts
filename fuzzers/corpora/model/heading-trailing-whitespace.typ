
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Whether headings contain trailing whitespace with or without comments/labels.
// Labels are special cased to immediately end headings in the parser, but also
// #strike[have unique whitespace behavior] Now their behavior is consistent!

#let join(..xs) = xs.pos().join()
#let head(h) = heading(depth: 1, h)

// No whitespace.
#test(head[h], [= h])
#test(head[h], [= h/**/])
#test(head[h], [= h<a>])
#test(head[h], [= h/**/<b>])

// #strike[Label behaves differently than normal trailing space and comment.]
// Now they behave the same!
#test(join(head[h])[ ], [= h  ])
#test(join(head[h])[ ], [= h  /**/])
#test(join(head[h])[ ], [= h  <c>])

// Combinations.
#test(join(head[h])[ ][ ], [= h  /**/  ])
#test(join(head[h])[ ][ ], [= h  <d>  ])
#test(join(head[h])[ ], [= h  /**/<e>])
#test(join(head[h])[ ], [= h/**/  <f>])

// #strike[The first space attaches, but not the second] Now neither attaches!
#test(join(head(join[h]))[ ][ ], [= h  /**/  <g>])