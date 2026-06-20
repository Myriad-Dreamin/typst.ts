
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that we don't pick up a numbering unrelated to the counted element.
#set heading(numbering: "A)")
#set math.equation(numbering: "1.")
= Hello
$ 1 + 2 $ <eq>
#context counter(heading).display(at: <eq>)