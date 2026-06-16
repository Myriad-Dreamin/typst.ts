// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 300pt, width: 200pt)
#table(
  columns: (1fr, 1fr),
  rows: (1fr, 1fr, 1fr),
  align: center + horizon,
  // Warning: 3-7 the `path` function is deprecated, use `curve` instead
  path(
    fill: red,
    closed: true,
    ((0%, 0%), (4%, -4%)),
    ((50%, 50%), (4%, -4%)),
    ((0%, 50%), (4%, 4%)),
    ((50%, 0%), (4%, 4%)),
  ),
  // Warning: 3-7 the `path` function is deprecated, use `curve` instead
  path(
    fill: purple,
    stroke: 1pt,
    (0pt, 0pt),
    (30pt, 30pt),
    (0pt, 30pt),
    (30pt, 0pt),
  ),
  // Warning: 3-7 the `path` function is deprecated, use `curve` instead
  path(
    fill: blue,
    stroke: 1pt,
    closed: true,
    ((30%, 0%), (35%, 30%), (-20%, 0%)),
    ((30%, 60%), (-20%, 0%), (0%, 0%)),
    ((50%, 30%), (60%, -30%), (60%, 0%)),
  ),
  // Warning: 3-7 the `path` function is deprecated, use `curve` instead
  path(
    stroke: 5pt,
    closed: true,
    (0pt,  30pt),
    (30pt, 30pt),
    (15pt, 0pt),
  ),
  // Warning: 3-7 the `path` function is deprecated, use `curve` instead
  path(
    fill: red,
    fill-rule: "non-zero",
    closed: true,
    (25pt, 0pt),
    (10pt, 50pt),
    (50pt, 20pt),
    (0pt, 20pt),
    (40pt, 50pt),
  ),
  // Warning: 3-7 the `path` function is deprecated, use `curve` instead
  path(
    fill: red,
    fill-rule: "even-odd",
    closed: true,
    (25pt, 0pt),
    (10pt, 50pt),
    (50pt, 20pt),
    (0pt, 20pt),
    (40pt, 50pt),
  ),
)
