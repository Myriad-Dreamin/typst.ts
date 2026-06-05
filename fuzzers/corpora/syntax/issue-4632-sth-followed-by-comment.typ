
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test heading markers followed by comments.
#test([
  =// Comment
  =/* Comment */
], [
  =
  =
])

// Test list markers followed by comments.
#test([
  -// Comment
  -/* Comment */
], [
  -
  -
])

// Test enum markers followed by comments.
#test([
  +// Comment
  +/* Comment */

  1.// Comment
  2./* Comment */
], [
  +
  +

  1.
  2.
])