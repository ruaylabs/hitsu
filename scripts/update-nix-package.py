import hashlib
import os
import re
from pathlib import Path

version = os.environ["RELEASE_TAG"][1:]
package = Path("nix/package.nix")
source = package.read_text()
version_pattern = r'(  version = ")([0-9]+\.[0-9]+\.[0-9]+)(";)'
current = re.search(version_pattern, source)

if current is None:
    raise SystemExit("Expected exactly one package version")
if tuple(map(int, version.split("."))) < tuple(map(int, current[2].split("."))):
    raise SystemExit("Refusing to downgrade the Nix package")

artifact = Path(os.environ["ARTIFACT_DIR"]) / f"Hitsu_{version}_amd64.deb"

with artifact.open("rb") as installer:
    checksum = hashlib.file_digest(installer, "sha256").hexdigest()

source, versions = re.subn(version_pattern, lambda m: m[1] + version + m[3], source)
source, checksums = re.subn(
    r'(    sha256 = ")[0-9a-f]{64}(";)', lambda m: m[1] + checksum + m[2], source
)

if versions != 1 or checksums != 1:
    raise SystemExit("Expected exactly one package version and checksum")

package.write_text(source)
