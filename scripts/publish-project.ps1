# Build
pwsh -Command { Set-Location projects/highlighter && yarn build }
pwsh -Command { Set-Location projects/vite-plugin-typst && yarn build }
pwsh -Command { Set-Location projects/rustdoc-typst-demo && cargo publish --dry-run }
# Release
pwsh -Command { Set-Location projects/hexo-renderer-typst && npm publish }
pwsh -Command { Set-Location projects/rehype-typst && npm publish --access public }
pwsh -Command { Set-Location projects/vite-plugin-typst && npm publish --access public }
pwsh -Command { Set-Location projects/rustdoc-typst-demo && cargo publish }
pwsh -Command { Set-Location projects/highlighter && npm publish --access public }
