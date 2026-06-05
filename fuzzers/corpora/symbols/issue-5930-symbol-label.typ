
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#emoji.face<lab>
#context test(query(<lab>).first().text, "😀")