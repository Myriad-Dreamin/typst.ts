import re
import sys

def reqwest_template(use_tls):
  return f"""reqwest = {{ version = "^0.11", default-features = false, features = [
      "{use_tls}",
      "blocking",
      "multipart",
  ] }}"""

def main():
  with open('compiler/Cargo.toml', 'r') as f:
    content = f.read()

  if sys.argv[1] == 'aarch64-pc-windows-msvc' or sys.argv[1] == 'riscv64gc-unknown-linux-gnu':
    tmpl = reqwest_template('native-tls')
  else:
    tmpl = reqwest_template('rustls-tls')

  content = re.sub(r'# begin itarget-region.*?# end itarget-region', tmpl, content, flags=re.DOTALL)
  print(content)

  with open('compiler/Cargo.toml', 'w') as f:
    f.write(content)

if __name__ == '__main__':
  main()
