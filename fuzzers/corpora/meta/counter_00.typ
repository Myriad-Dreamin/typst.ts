
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Count with string key.
#let mine = counter("mine!")

Final: #locate(loc => mine.final(loc).at(0)) \
#mine.step()
First: #mine.display() \
#mine.update(7)
#mine.display("1 of 1", both: true) \
#mine.step()
#mine.step()
Second: #mine.display("I")
#mine.update(n => n * 2)
#mine.step()
