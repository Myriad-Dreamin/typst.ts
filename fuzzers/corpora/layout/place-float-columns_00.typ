
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 200pt, width: 300pt)
#show: columns.with(2)

= Introduction
#figure(
  placement: bottom,
  caption: [A glacier],
  image("/assets/files/glacier.jpg", width: 50%),
)
#lorem(45)
#figure(
  placement: top,
  caption: [A rectangle],
  rect[Hello!],
)
#lorem(20)
