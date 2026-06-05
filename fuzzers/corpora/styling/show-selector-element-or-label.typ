
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test element selector combined with label selector.
#show selector(strong).or(<special>): highlight
I am *strong*, I am _emphasized_, and I am #[special<special>].