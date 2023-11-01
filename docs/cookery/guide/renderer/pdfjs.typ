
=== Provides the instance of pdf.js library

Currently it is hardcoded by getting value of `window.pdfjsLib`. Example setup:

```html
<!-- pdf.js v3.5.141 -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.5.141/pdf.min.js"></script>
<link rel="stylesheet" href="/core/examples/typst.ts.css" />
<link
  rel="stylesheet"
  href="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.5.141/pdf_viewer.min.css"
  integrity="sha512-Jf9DLkegLgARLR151csVkPvcVt4cOUhslrSZwiTAe9mqFL/BbYRDmxCOioCtbHifEgjsBFbrVhOMQ3mYPDLrqQ=="
  crossorigin="anonymous"
  referrerpolicy="no-referrer"
  />
<script>
  pdfjsLib.GlobalWorkerOptions.workerSrc =
    'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.5.141/pdf.worker.min.js';
</script>
```

We will remove dependency on pdf.js in future.
