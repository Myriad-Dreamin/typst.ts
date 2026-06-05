
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that when a breakable element fully migrates to the next page without
// orphan frames, its position correctly reflects that.
#set page(height: 40pt)
A
#block[B]<b>

#context test(
  locate(<b>).position(),
  (page: 2, x: 10pt, y: 10pt),
)