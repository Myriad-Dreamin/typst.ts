
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test locating tags that are before or between pages.
#set page(height: 30pt)
#context [
  // Before the first page.
  // (= at the very start of the first page, before the header)
  #test(locate(<a>).position(), (page: 1, x: 0pt, y: 0pt))

  // On the first page.
  #test(locate(<b>).position(), (page: 1, x: 10pt, y: 10pt))

  // Between the two pages.
  // (= at the very start of the first page, before the header)
  #test(locate(<c>).position(), (page: 2, x: 0pt, y: 0pt))

  // After the last page.
  // (= at the very end of the last page, after the footer)
  #test(locate(<d>).position(), (page: 2, x: 0pt, y: 30pt))
  #test(locate(<e>).position(), (page: 2, x: 0pt, y: 30pt))
]

#metadata(none) <a>
#pagebreak(weak: true)
#metadata(none) <b>
A
#pagebreak()
#metadata(none) <c>
#pagebreak(weak: true)
B
#pagebreak(weak: true)
#metadata(none) <d>
#pagebreak(weak: true)
#metadata(none) <e>