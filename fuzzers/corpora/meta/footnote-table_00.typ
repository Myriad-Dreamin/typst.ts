
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page


#set page(height: 100pt)

= Tables
#table(
  columns: 2,
  [Hello footnote #footnote[This is a footnote.]],
  [This is more text],
  [This cell
   #footnote[This footnote is not on the same page]
   breaks over multiple pages.],
  image("/assets/files/tiger.jpg"),
)

#table(
  columns: 3,
  ..range(1, 10)
    .map(numbering.with("a"))
    .map(v => upper(v) + footnote(v))
)
