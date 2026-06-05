
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Tab chars were not rendered in raw blocks with lang: "typ(c)"
#raw("#if true {\n\tf()\t// typ\n}", lang: "typ")

#raw("if true {\n\tf()\t// typc\n}", lang: "typc")

```typ
#if true {
	// tabs around f()
	f()	// typ
}
```

```typc
if true {
	// tabs around f()
	f()	// typc
}
```