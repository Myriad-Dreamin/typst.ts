# Troubleshooting WASM Testing

### ChromeDriver reports that `Error: non-200 response code: 404`

If you encounter the following error when running a wasm test:

```
Error: non-200 response code: 404
{"value":{"error":"invalid session id","message":"invalid session id","stacktrace":"Backtrace:\n\tGetHandleVerifier ...
```

It may be caused by a version mismatch between `chromedriver` and `google-chrome`. Please ensure that they are on the same version to resolve this issue. You can check this by:

```
# Output of wasm test
driver stdout:
    Starting ChromeDriver 114.0.5735.90
# Output of open chrome://version in browser
Google Chrome: 114.0.5735.90
```

If there is a version mismatch between the two, update `chromedriver` or `google-chrome` accordingly.

Related issue: https://github.com/rustwasm/wasm-bindgen/issues/2151
