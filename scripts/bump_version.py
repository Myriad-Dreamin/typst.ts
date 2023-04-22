import sys
import os


def main(old_version, new_version):
  for v in [old_version, new_version]:
    if len(v.split('.')) != 3:
      raise ValueError('Version must be in the form x.y.z')

  for file_path in [
      "cli/Cargo.toml",
      "core/Cargo.toml",
      "compiler/Cargo.toml",
      "packages/typst.ts/Cargo.toml",
      "exporter/ws/Cargo.toml",
      "contrib/fontctl/Cargo.toml",
      "contrib/fontctl/src/main.rs",
      "cli/src/lib.rs",
  ]:
    with open(file_path) as f:
      content = f.read()

    new_content = content.replace(f'version = "{old_version}"', f'version = "{new_version}"')
    if content == new_content:
      if f'version = "{new_version}"' in content:
        print(f'Version in {file_path} already set to {new_version}')
        continue

      raise ValueError(
        f'Failed to replace version in {file_path} from {old_version} to {new_version}')

    with open(file_path, 'w') as f:
      f.write(new_content)

  for package_file in [
      "packages/typst.ts/package.json",
      "packages/typst.react/package.json",
  ]:
    with open(package_file) as f:
      content = f.read()

    new_content = content.replace(f'"version": "{old_version}"', f'"version": "{new_version}"')
    if content == new_content:
      if f'"{new_version}"' in content:
        print(f'Version in {package_file} already set to {new_version}')
        continue

      raise ValueError(
        f'Failed to replace version in {package_file} from {old_version} to {new_version}')

    with open(package_file, 'w') as f:
      f.write(new_content)


if __name__ == '__main__':
  main(sys.argv[1], sys.argv[2])
