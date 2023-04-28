import sys
import itertools


def replace_version(package_file, old_version, new_version):
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


def main(old_version, new_version):
  for v in [old_version, new_version]:
    if len(v.split('.')) != 3:
      raise ValueError('Version must be in the form x.y.z')

  for file_path, version_form in itertools.product([
      "cli/Cargo.toml",
      "core/Cargo.toml",
      "compiler/Cargo.toml",
      "contrib/fontctl/Cargo.toml",
      "contrib/dev-server/Cargo.toml",
      "exporter/canvas/Cargo.toml",
      "exporter/pdf/Cargo.toml",
      "exporter/raster/Cargo.toml",
      "exporter/ws/Cargo.toml",
      "packages/typst.ts/Cargo.toml",
  ], [
      lambda v: f'version = "{v}"',
      lambda v: f'typst-ts-core = "{v}"',
      lambda v: f'typst-ts-compiler = "{v}"',
  ]):
    replace_version(file_path, version_form(old_version), version_form(new_version))

  def version_lit(v):
    return f'"version": "{v}"'

  def dep_core_lit(v):
    return f'"@myriaddreamin/typst.ts": "^{v}"'

  for package_file in [
      "packages/typst.ts/package.json",
      "packages/typst.react/package.json",
  ]:
    replace_version(package_file, version_lit(old_version), version_lit(new_version))
    if 'typst.ts' not in package_file:
      replace_version(package_file, dep_core_lit(old_version), dep_core_lit(new_version))


if __name__ == '__main__':
  main(sys.argv[1], sys.argv[2])
