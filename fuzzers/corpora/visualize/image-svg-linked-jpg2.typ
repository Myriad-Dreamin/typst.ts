
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(fill: gray)
#image(bytes(
  ```
  <svg xmlns="http://www.w3.org/2000/svg" height="80" width="48">
    <image href="file://../../../assets/images/f2t.jpg" />
    <circle r="32" cx="24" cy="40" fill="none" stroke="blue" />
  </svg>
  ```.text
))