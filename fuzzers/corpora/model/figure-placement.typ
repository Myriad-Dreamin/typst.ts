
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 160pt, columns: 2)
#set place(clearance: 10pt)

#lines(4)

#figure(
  placement: auto,
  scope: "parent",
  caption: [I],
  rect(height: 15pt, width: 80%),
)

#figure(
  placement: bottom,
  caption: [II],
  rect(height: 15pt, width: 80%),
)

#lines(2)

#figure(
  placement: bottom,
  caption: [III],
  rect(height: 25pt, width: 80%),
)

#figure(
  placement: auto,
  scope: "parent",
  caption: [IV],
  rect(width: 80%),
)

#lines(15)