
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(first-line-indent: 12pt, spacing: 5pt, leading: 5pt)
#show heading: set text(size: 10pt)

The first paragraph has no indent.

But the second one does.

#box(image("/assets/images/tiger.jpg", height: 6pt))
starts a paragraph, also with indent.

#align(center, image("/assets/images/rhino.png", width: 1cm))

= Headings
- And lists.
- Have no indent.

  Except if you have another paragraph in them.

#set text(8pt, lang: "ar", font: ("Noto Sans Arabic", "Libertinus Serif"))
#set par(leading: 8pt)

= Arabic
دع النص يمطر عليك

ثم يصبح النص رطبًا وقابل للطرق ويبدو المستند رائعًا.