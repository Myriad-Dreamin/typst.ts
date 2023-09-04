
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Check that unbalanced brackets are not included in links.
#[https://example.com/] \
https://example.com/)
