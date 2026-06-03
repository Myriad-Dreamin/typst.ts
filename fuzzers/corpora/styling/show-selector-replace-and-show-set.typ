
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test full reset.
#show heading: [B]
#show heading: set text(size: 10pt, weight: 400)
A #[= Heading] C