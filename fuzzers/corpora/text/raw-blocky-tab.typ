
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let raw = {
```
	test
```
}
#test(raw.text, "\ttest")
#test(raw.block, true)