
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test fields on elements.
#show list: it => {
  test(it.children.len(), 3)
}

- A
- B
- C