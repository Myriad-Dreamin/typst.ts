
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set rect(
  width: 100%,
  height: 20pt,
  fill: tiling(relative: "parent", circle(radius: 10pt)),
)
#stack(spacing: 5pt, rect(), rect())