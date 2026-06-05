
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#pdf.attach("/assets/text/hello.txt")
#pdf.attach(
  "/assets/data/details.toml",
  relationship: "supplement",
  mime-type: "application/toml",
  description: "Information about a secret project",
)