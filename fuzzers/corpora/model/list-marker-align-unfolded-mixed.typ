
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Verify whether overriding vertical alignment causes horizontal alignment to
// be inherited from the context.
#set align(center)
#set list(
  marker-align: top,
  marker: {
    // Artificially cause markers to have a different width.
    counter("b").step()
    context {
      "1" * counter("b").get().first()
    }
  }
)

- abc
- abc
- abc