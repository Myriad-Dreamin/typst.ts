
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 100pt)
#stack(dir: ltr,
  align(left, square(size: 15pt, fill: eastern)),
  align(center, square(size: 20pt, fill: eastern)),
  align(right, square(size: 15pt, fill: eastern)),
)
#align(center + horizon, rect(fill: eastern, height: 10pt))
#align(bottom, stack(
  align(center, rect(fill: conifer, height: 10pt)),
  rect(fill: forest, height: 10pt, width: 100%),
))
