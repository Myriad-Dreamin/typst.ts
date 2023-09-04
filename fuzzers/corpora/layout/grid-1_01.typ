
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set rect(inset: 0pt)
#grid(
  columns: (auto, auto, 40%),
  column-gutter: 1fr,
  row-gutter: 1fr,
  rect(fill: eastern)[dddaa aaa aaa],
  rect(fill: conifer)[ccc],
  rect(fill: rgb("dddddd"))[aaa],
)
