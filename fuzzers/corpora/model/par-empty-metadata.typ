
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Check that metadata still works in a zero length paragraph.
#block(height: 0pt)[#""#metadata(false)<hi>]
#context test(query(<hi>).first().value, false)