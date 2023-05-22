import sys
import itertools


def replace_version(package_file: str, old_version: str, new_version: str):
  with open(package_file) as f:
    content = f.read()

  new_content = content.replace(old_version, new_version)
  if content == new_content:
    if new_version in content:
      print(f'Version in {package_file} already set to {new_version}')
      return
    if old_version not in content:
      if 'version' in new_version:
        print(f'Version in {package_file} is not found, did not set to {new_version}')
      return

    raise ValueError(
      f'Failed to replace version in {package_file} from {old_version} to {new_version}')

  with open(package_file, 'w') as f:
    f.write(new_content)


def main(old_version: str, new_version: str):
  # validate version format
  for v in [old_version, new_version]:
    if len(v.split('.')) != 3:
      raise ValueError(f'Version String "{v}" must be in the form x.y.z')

  bump_rust_self_version = lambda: itertools.product(
    [ # file paths
      "Cargo.toml",
      "exporter/ast/Cargo.toml",
      "exporter/canvas/Cargo.toml",
      "exporter/pdf/Cargo.toml",
      "exporter/raster/Cargo.toml",
      "exporter/serde/Cargo.toml",
      "exporter/ws/Cargo.toml",
      "exporter/tir/Cargo.toml",
    ], [ # patterns
      lambda v: f'version = "{v}"',
    ])

  bump_javascript_self_version = lambda: itertools.product(
    [ # file paths
      "packages/compiler/package.json",
      "packages/renderer/package.json",
      "packages/typst.ts/package.json",
      "packages/typst.react/package.json",
      "packages/typst.angular/projects/typst.angular/package.json",
    ], [ # patterns
      lambda v: f'"version": "{v}"',
    ])
  
  bump_self_version = lambda: itertools.chain(
    bump_rust_self_version(),
    bump_javascript_self_version(),
  )

  for file_path, pattern in itertools.chain(bump_self_version()):
    replace_version(file_path, pattern(old_version), pattern(new_version))


if __name__ == '__main__':
  main(
    # old version
    sys.argv[1],
    # new version
    sys.argv[2])
