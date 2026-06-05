
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
  </head>
  <body>
    <h1>Topic</h1>
    <p>The Hypertext Markup Language.</p>
    <script>
      function foo(a, b) {
        return a + b + "string";
      }
    </script>
  </body>
</html>
```