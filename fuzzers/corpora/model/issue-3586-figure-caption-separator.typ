
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that figure caption separator is synthesized correctly.
#show figure.caption: c => test(c.separator, [#": "])
#figure(table[], caption: [This is a test caption])