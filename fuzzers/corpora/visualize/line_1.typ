// Test lines.

#set page(height: 60pt)
#box({
  set line(stroke: 0.75pt)
  place(line(end: (0.4em, 0pt)))
  place(line(start: (0pt, 0.4em), end: (0pt, 0pt)))
  line(end: (0.6em, 0.6em))
}) Hello #box(line(length: 1cm))!

#line(end: (70%, 50%))
