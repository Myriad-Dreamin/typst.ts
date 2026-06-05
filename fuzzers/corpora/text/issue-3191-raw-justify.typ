
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Raw blocks should not be justified by default.
```
a b c --------------------
```

#show raw: set par(justify: true)
```
a b c --------------------
```