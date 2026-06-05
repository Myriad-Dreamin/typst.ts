
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show heading: it => {
  metadata(it.label)
  pagebreak(weak: true, to: "odd")
  it
}

Hi
= Hello <hello>
= World <world>

// The metadata's position does not migrate to the next page, but the heading's
// does.
#context {
  test(locate(metadata.where(value: <hello>)).page(), 1)
  test(locate(<hello>).page(), 3)
  test(locate(metadata.where(value: <world>)).page(), 3)
  test(locate(<world>).page(), 5)
}