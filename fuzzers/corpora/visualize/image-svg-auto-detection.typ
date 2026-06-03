
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#image(bytes(
  ```
  <?xml version="1.0" encoding="utf-8"?>
  <!-- An SVG -->
  <svg width="200" height="150" xmlns="http://www.w3.org/2000/svg">
    <rect fill="red" stroke="black" x="25" y="25" width="150" height="100"/>
  </svg>
  ```.text
))