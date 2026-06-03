
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test locating the position of a tag with no following content.
#context test(here().position().y, 10pt)
#box[]
#v(10pt)
#context test(here().position().y, 20pt)