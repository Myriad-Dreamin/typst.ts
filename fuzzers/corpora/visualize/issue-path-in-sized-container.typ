
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Paths used to implement `LayoutMultiple` rather than `LayoutSingle` without
// fulfilling the necessary contract of respecting region expansion.
#block(
  fill: aqua,
  width: 20pt,
  height: 15pt,
  // Warning: 3-7 the `path` function is deprecated, use `curve` instead
  path(
    (0pt, 0pt),
    (10pt, 10pt),
  ),
)