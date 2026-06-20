
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// We used to not honor resvg's `exclude_fonts` mechanism, which could result in
// an infinite fallback loop.
//
// On way to trigger this is if there are two codepoints that result in one
// cluster, and the first exists in a font, but the second always shapes to a
// tofu.
#image(bytes(
  ```
  <svg xmlns="http://www.w3.org/2000/svg" height="1" width="1">
    <text font-family="Libertinus Serif">x&#1761;</text>
  </svg>
  ```.text
))