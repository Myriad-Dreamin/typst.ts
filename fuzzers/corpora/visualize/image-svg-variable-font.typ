
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#image(bytes(
  ```
  <svg id="svg1" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <text
      x="10" y="40" font-family="Cantarell" font-size="32"
      style="font-variation-settings: 'wght' 300"
    >
      Hello
    </text>
    <text
      x="10" y="80" font-family="Cantarell" font-size="32"
      style="font-variation-settings: 'wght' 700"
    >
      Hello
    </text>
  </svg>
  ```.text
))