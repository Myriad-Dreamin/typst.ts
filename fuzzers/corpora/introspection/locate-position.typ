
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test `locate`.
#v(10pt)
= Introduction <intro>
#context test(locate(<intro>).position().y, 20pt)