
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let raw = eval("```   \n```")
#test(raw.text, "")
#test(raw.block, true)