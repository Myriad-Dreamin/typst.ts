
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that tags and spaces aren't reordered in textual grouping.
A#metadata(none)<a> #metadata(none)<b>#box[B]

#context assert(
  locate(<a>).position().x + 1pt < locate(<b>).position().x
)