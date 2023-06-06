module.exports = function (_locals) {
  return `
    <script src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.5.141/pdf.min.js"></script>
    <script>
        pdfjsLib.GlobalWorkerOptions.workerSrc =
        'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.5.141/pdf.worker.min.js';
    </script>
    <link
        rel="stylesheet"
        href="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.5.141/pdf_viewer.min.css"
        integrity="sha512-Jf9DLkegLgARLR151csVkPvcVt4cOUhslrSZwiTAe9mqFL/BbYRDmxCOioCtbHifEgjsBFbrVhOMQ3mYPDLrqQ=="
        crossorigin="anonymous"
        referrerpolicy="no-referrer"
    />`;
};
