
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#pdf.attach("hello.txt", read("/assets/text/hello.txt", encoding: none))
#pdf.attach(
  "a_file_name.txt",
  read("/assets/text/hello.txt", encoding: none),
  relationship: "supplement",
  mime-type: "text/plain",
  description: "A description",
)