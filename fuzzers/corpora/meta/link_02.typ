
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Verify that brackets are included in links.
https://[::1]:8080/ \
https://example.com/(paren) \
https://example.com/#(((nested))) \
