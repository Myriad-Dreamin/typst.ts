import hashlib
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location(
    "install_tool", Path(__file__).with_name("install-tool.py")
)
installer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(installer)


def archive_with(*names):
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w:gz") as archive:
        for name in names:
            member = tarfile.TarInfo(name)
            member.size = 6
            archive.addfile(member, io.BytesIO(b"binary"))
    return buffer.getvalue()


class InstallToolTests(unittest.TestCase):
    def install(self, data, checksum, directory):
        tool = ("https://example.invalid/tool.tar.gz", checksum, "tool")
        with (
            patch.dict(installer.TOOLS, {"test": tool}),
            patch.object(installer.urllib.request, "urlopen", return_value=io.BytesIO(data)),
        ):
            return installer.install("test", directory)

    def test_checksum_mismatch_writes_nothing(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "bin"
            with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
                self.install(b"tampered", "0" * 64, target)
            self.assertFalse(target.exists())

    def test_only_selected_binary_is_written_without_archive_paths(self):
        data = archive_with("../../tool", "../../unwanted")
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "bin"
            binary = self.install(data, hashlib.sha256(data).hexdigest(), target)
            self.assertEqual(binary, target / "tool")
            self.assertEqual(binary.read_bytes(), b"binary")
            self.assertEqual(list(target.iterdir()), [binary])

    def test_duplicate_binary_is_rejected(self):
        data = archive_with("one/tool", "two/tool")
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "Expected exactly one"):
                self.install(data, hashlib.sha256(data).hexdigest(), Path(directory))


if __name__ == "__main__":
    unittest.main()
