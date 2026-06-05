
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test folding.
#set rect(stroke: red)
#context {
  test(type(rect.stroke), stroke)
  test(rect.stroke.paint, red)
}
#[
  #set rect(stroke: 4pt)
  #context test(rect.stroke, 4pt + red)
]
#context test(rect.stroke, stroke(red))