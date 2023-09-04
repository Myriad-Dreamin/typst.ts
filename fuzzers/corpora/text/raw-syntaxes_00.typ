
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 180pt)
#set text(6pt)
#set raw(syntaxes: "/assets/files/SExpressions.sublime-syntax")

```sexp
(defun factorial (x)
  (if (zerop x)
    ; with a comment
    1
    (* x (factorial (- x 1)))))
```
