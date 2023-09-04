
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Set width and height.
// Should result in one high and one wide page.
#set page(width: 80pt, height: 80pt)
#[#set page(width: 40pt);High]
#[#set page(height: 40pt);Wide]

// Flipped predefined paper.
#[#set page(paper: "a11", flipped: true);Flipped A11]
