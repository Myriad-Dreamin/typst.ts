#import "/docs/cookery/book.typ": book-page

#show: book-page.with(title: "Hexo Library")

= Hexo Library

Add `"hexo-renderer-typst": "0"` to your `package.json`

And run:

```
# serve files
hexo serve
# generate files
hexo generate
```

Currently, it could only render typst documents inside of `source/_posts` (Hexo Posts) and fix typst workspace (root directory) to the root of your blog project.
