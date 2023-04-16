### In-place Replacement Plugin for MathJax Users

Example code:

```html
<head>
  <script type="module">
    import { TypstJax } from '@myriaddreamin/typst-math';
    TypstJax.Config = {
      tex: {
        inlineMath: [
          ['$', '$'],
          ['\\(', '\\)'],
        ],
      },
      svg: {
        fontCache: 'global',
      },
    };
  </script>
</head>
<body>
  $$mono(Y) = lambda f. (lambda x. f (x space x)) (lambda x . f (x space x))$$
</body>
```
