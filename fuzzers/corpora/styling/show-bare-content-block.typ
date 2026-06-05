
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test bare show in content block.
A #[_B #show: c => [*#c*]; C_] D