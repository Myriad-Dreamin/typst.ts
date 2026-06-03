
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(
  paper: "a8",
  margin: (x: 15pt, y: 30pt),
  header: {
    text(eastern)[*Typst*]
    h(1fr)
    text(0.8em)[_Chapter 1_]
  },
  footer: context align(center)[\~ #counter(page).display() \~],
  background: context if counter(page).get().first() <= 2 {
    place(center + horizon, circle(radius: 1cm, fill: luma(90%)))
  }
)

#align(center, lines(20))

#set page(header: none, height: auto, margin: (top: 15pt, bottom: 25pt))
Z