
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show par: highlight

I'm a paragraph.

#align(center, table(
  columns: 3,

  // No paragraphs.
  [A],
  block[B],
  block[C *D*],

  // Paragraphs.
  par[E],
  [

    F
  ],
  [
    G

  ],

  // Paragraphs.
  parbreak() + [H],
  [I] + parbreak(),
  parbreak() +  [J] + parbreak(),

  // Paragraphs.
  [K #v(10pt)],
  [#v(10pt) L],
  [#place[] M],

  // Paragraphs.
  [
    N

    O
  ],
  [#par[P]#par[Q]],
  // No paragraphs.
  [#block[R]#block[S]],
))