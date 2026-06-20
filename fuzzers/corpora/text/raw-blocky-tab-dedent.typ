
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This one is a bit problematic because there is a trailing tab below "test"
// which the editor constantly wants to remove.
#let raw = eval("```\n\ttest\n  \n ```")
#test(raw.text, "test\n ")
#test(raw.block, true)