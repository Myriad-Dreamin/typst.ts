
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that document set rule can be conditional on document information
// itself.
#set document(author: "Normal", title: "Alternative")
#context {
  set document(author: "Changed") if "Normal" in document.author
  set document(title: "Changed") if document.title ==  "Normal"
}