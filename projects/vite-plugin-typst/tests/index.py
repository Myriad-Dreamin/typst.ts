import subprocess


def run_vite_on(project_path):
  print(f"Running vite on {project_path}")
  return subprocess.check_call(
    "npx vite build",
    cwd=project_path,
    shell=True,
  )


def main():
  for project, expected_ok in [
    ["examples/single-file", True],
    ["examples/single-file-error", False],
    ["examples/glob-documents", True],
    ["examples/single-file", True],
    ["examples/query", True],
    ["examples/js-import", True],
    ["examples/mixin-parts", True],
  ]:
    try:
      code = run_vite_on(project)
      if code != 0 and not expected_ok:
        raise Exception(f"Expected error in {project}")
    except subprocess.CalledProcessError as err:
      if expected_ok:
        raise Exception(f"Unexpected error in {project}: {err}")


if __name__ == "__main__":
  main()
