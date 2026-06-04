
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#pdf.attach(
  "/assets/text/hello.txt",
  relationship: "supplement",
  mime-type: "text/plain",
  description: "Reference attachment for PDF comparison",
)

#pdf.artifact(kind: "page")[
  #rect(width: 100%, height: 6pt, fill: luma(230))
]

#strong[Compare PDF]

Attachment and artifact metadata should not change the rendered page.
