#!/usr/bin/env python3
# Used this script as reference: https://github.com/artichoke/nightly/blob/trunk/macos_sign_and_notarize.py
# Based on Apple documentation, we want the binaries to be signed AND notarized
#   https://support.apple.com/en-us/HT202491

import argparse
import base64
import binascii
import json
import os
import secrets
import shutil
import subprocess
import sys
import tempfile
import traceback
import urllib.request
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Optional

MACOS_SIGN_AND_NOTARIZE_VERSION = "0.3.1"


def run_command_with_merged_output(command: list[str]) -> None:
    """
    Run the given command as a subprocess and merge its stdout and stderr
    streams.

    This is useful for funnelling all output of a command into a GitHub Actions
    log group.

    This command uses `check=True` when delegating to `subprocess`.
    """

    proc = subprocess.run(
        command,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    for line in proc.stdout.splitlines():
        if line:
            print(line)


def set_output(*, name: str, value: str) -> None:
    """
    Set an output for a GitHub Actions job.

    https://docs.github.com/en/actions/using-jobs/defining-outputs-for-jobs
    https://github.blog/changelog/2022-10-11-github-actions-deprecating-save-state-and-set-output-commands/
    """

    if github_output := os.getenv("GITHUB_OUTPUT"):
        with open(github_output, "a") as out:
            print(f"{name}={value}", file=out)


@contextmanager
def log_group(group: str) -> Iterator[None]:
    """
    Create an expandable log group in GitHub Actions job logs.

    https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#grouping-log-lines
    """

    print(f"::group::{group}")
    try:
        yield
    finally:
        print("::endgroup::")


@contextmanager
def attach_disk_image(image: Path, *, readwrite: bool = False) -> Iterator[Path]:
    try:
        with log_group("Attaching disk image"):
            if readwrite:
                command = [
                    "/usr/bin/hdiutil",
                    "attach",
                    "-readwrite",
                    "-noverify",
                    "-noautoopen",
                    str(image),
                ]
            else:
                command = ["/usr/bin/hdiutil", "attach", str(image)]
            run_command_with_merged_output(command)

        mounted_image = disk_image_mount_path()
        yield mounted_image
    finally:
        with log_group("Detaching disk image"):
            run_command_with_merged_output(
                ["/usr/bin/hdiutil", "detach", str(mounted_image)],
            )


def get_image_size(image: Path) -> int:
    """
    Compute the size in megabytes of a disk image.

    This method is influenced by `create-dmg`:
    https://github.com/create-dmg/create-dmg/blob/412e99352bacef0f05f9abe6cc4348a627b7ac56/create-dmg#L306-L315
    """

    proc = subprocess.run(
        ["/usr/bin/sw_vers", "-productVersion"],
        check=True,
        capture_output=True,
        text=True,
    )
    major, *_rest = proc.stdout.strip().split(".", 1)

    if int(major) >= 12:
        proc = subprocess.run(
            ["/usr/bin/du", "-B", "512", "-s", str(image)],
            check=True,
            capture_output=True,
            text=True,
        )
        size = int(proc.stdout.split()[0])
    else:
        proc = subprocess.run(
            ["/usr/bin/du", "-s", str(image)],
            check=True,
            capture_output=True,
            text=True,
        )
        size = int(proc.stdout.split()[0])

    return (size * 512 // 1000 // 1000) + 1


def emit_metadata() -> None:
    if os.getenv("CI") != "true":
        return
    with log_group("Workflow metadata"):
        if repository := os.getenv("GITHUB_REPOSITORY"):
            print(f"GitHub Repository: {repository}")
        if actor := os.getenv("GITHUB_ACTOR"):
            print(f"GitHub Actor: {actor}")
        if workflow := os.getenv("GITHUB_WORKFLOW"):
            print(f"GitHub Workflow: {workflow}")
        if job := os.getenv("GITHUB_JOB"):
            print(f"GitHub Job: {job}")
        if run_id := os.getenv("GITHUB_RUN_ID"):
            print(f"GitHub Run ID: {run_id}")
        if ref := os.getenv("GITHUB_REF"):
            print(f"GitHub Ref: {ref}")
        if ref_name := os.getenv("GITHUB_REF_NAME"):
            print(f"GitHub Ref Name: {ref_name}")
        if sha := os.getenv("GITHUB_SHA"):
            print(f"GitHub SHA: {sha}")


def keychain_path() -> Path:
    """
    Absolute path to a keychain used for the codesigning and notarization
    process.

    This path is a constant.
    """

    # If running on GitHub Actions, use the `RUNNER_TEMP` directory.
    #
    # `RUNNER_TEMP` is the path to a temporary directory on the runner. This
    # directory is emptied at the beginning and end of each job.
    if runner_temp := os.getenv("RUNNER_TEMP"):
        return Path(runner_temp).joinpath("notarization.keychain-db")
    else:
        return Path("notarization.keychain-db").resolve()


def notarytool_credentials_profile() -> str:
    """
    Name of the credentials profile stored in the build keychain for use with
    notarytool.

    This profile is a constant.
    """

    return "some-apple-codesign-notary"


def codesigning_identity() -> str:
    """
    Codesigning identity and name of the Apple Developer ID Application.
    """

    return "Developer ID Application: $FirstName $LastName ($TEAMID)"


def notarization_apple_id() -> str:
    """
    Apple ID belonging to the codesigning identity.
    """

    return "apple-codesign@mobilecoin.com"


def notarization_app_specific_password() -> str:
    """
    App-specific password for the notarization process belonging to the
    codesigning identity's Apple ID.
    """

    if app_specific_password := os.getenv("MACOS_NOTARIZE_APP_PASSWORD"):
        return app_specific_password
    raise Exception("MACOS_NOTARIZE_APP_PASSWORD environment variable is required")


def notarization_team_id() -> str:
    """
    Team ID belonging to the codesigning identity.
    """

    return "$TEAMID"


def disk_image_volume_name() -> str:
    """
    Volume name for the newly created DMG disk image.
    """

    return os.getenv("BUILDNAME")


def disk_image_mount_path() -> Path:
    """
    Mount path for the newly created DMG disk image.
    """

    return Path("/Volumes").joinpath(disk_image_volume_name())


def create_keychain(*, keychain_password: str) -> None:
    """
    Create a new keychain for the codesigning and notarization process.

    This ephemeral keychain stores Apple ID credentials for `notarytool` and
    code signing certificates for `codesign`.
    """

    # Ensure keychain does not exist.
    delete_keychain()

    with log_group("Setup notarization keychain"):
        # security create-keychain -p "$keychain_password" "$keychain_path"
        run_command_with_merged_output(
            [
                "security",
                "create-keychain",
                "-p",
                keychain_password,
                str(keychain_path()),
            ]
        )
        print(f"Created keychain at {keychain_path()}")

        # security set-keychain-settings -lut 900 "$keychain_path"
        run_command_with_merged_output(
            ["security", "set-keychain-settings", "-lut", "900", str(keychain_path())]
        )
        print("Set keychain to be ephemeral")

        # security unlock-keychain -p "$keychain_password" "$keychain_path"
        run_command_with_merged_output(
            [
                "security",
                "unlock-keychain",
                "-p",
                keychain_password,
                str(keychain_path()),
            ]
        )
        print(f"Unlocked keychain at {keychain_path()}")

        # Per `man codesign`, the keychain filename passed via the `--keychain`
        # argument will not be searched to resolve the signing identity's
        # certificate chain unless it is also on the user's keychain search list.
        #
        # `security create-keychain` does not add keychains to the search path.
        # _Opening_ them does, as well as explicitly manipulating the search path
        # with `security list-keychains -s`.
        #
        # This stackoverflow post explains the solution:
        # <https://stackoverflow.com/a/49640952>
        #
        # `security delete-keychain` removes the keychain from the search path.
        proc = subprocess.run(
            ["security", "list-keychains", "-d", "user"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )

        search_path = [line.strip().strip('"') for line in proc.stdout.splitlines()]
        search_path = [keychain for keychain in search_path if keychain]
        search_path.append(str(keychain_path()))

        run_command_with_merged_output(
            ["security", "list-keychains", "-d", "user", "-s"] + search_path
        )
        print(f"Set keychain search path: {', '.join(search_path)}")


def delete_keychain() -> None:
    """
    Delete the keychain for the codesigning and notarization process.

    This ephemeral keychain stores Apple ID credentials for `notarytool` and
    code signing certificates for `codesign`.
    """

    with log_group("Delete keychain"):
        # security delete-keychain /path/to/notarization.keychain-db
        proc = subprocess.run(
            ["security", "delete-keychain", str(keychain_path())],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        for line in proc.stdout.splitlines():
            print(line)

        if proc.returncode == 0:
            print(f"Keychain deleted from {keychain_path()}")
        else:
            # keychain does not exist
            print(f"Keychain not found at {keychain_path()}, ignoring ...")


def import_notarization_credentials() -> None:
    """
    Import credentials required for notarytool to the codesigning and notarization
    process keychain.

    See `notarytool_credentials_profile`, `notarization_apple_id`,
    `notarization_app_specific_password`, and `notarization_team_id`.
    """

    with log_group("Import notarization credentials"):
        # xcrun notarytool store-credentials \
        #   "$notarytool_credentials_profile" \
        #   --apple-id "apple-codesign@artichokeruby.org" \
        #   --password "$MACOS_NOTARIZE_APP_PASSWORD" \
        #   --team-id "VDKP67932G" \
        #   --keychain "$keychain_path"
        run_command_with_merged_output(
            [
                "/usr/bin/xcrun",
                "notarytool",
                "store-credentials",
                notarytool_credentials_profile(),
                "--apple-id",
                notarization_apple_id(),
                "--password",
                notarization_app_specific_password(),
                "--team-id",
                notarization_team_id(),
                "--keychain",
                str(keychain_path()),
            ],
        )


def import_certificate(
    *, path: Path, name: Optional[str] = None, password: Optional[str] = None
) -> None:
    """
    Import a certificate at a given path into the build keychain.
    """

    # security import certificate.p12 \
    #   -k "$keychain_path" \
    #   -P "$MACOS_CERTIFICATE_PWD" \
    #   -T /usr/bin/codesign
    command = [
        "security",
        "import",
        str(path),
        "-k",
        str(keychain_path()),
        "-T",
        "/usr/bin/codesign",
    ]
    if password is not None:
        command.extend(["-P", password])

    run_command_with_merged_output(command)

    cert_name = path if name is None else name
    print(f"Imported certificate {cert_name}")


def import_codesigning_certificate() -> None:
    """
    Import codesigning certificate into the codesigning and notarization process
    keychain.

    The certificate is expected to be a base64-encoded string stored in the
    `MACOS_CERTIFICATE` environment variable with a password given by the
    `MACOS_CERTIFICATE_PASSPHRASE` environment variable.

    The base64-encoded certificate is stored in a temporary file so it may be
    imported into the keychain by the `security` utility.
    """

    with log_group("Import codesigning certificate"):
        encoded_certificate = os.getenv("MACOS_CERTIFICATE")
        if not encoded_certificate:
            raise Exception("MACOS_CERTIFICATE environment variable is required")

        try:
            certificate = base64.b64decode(encoded_certificate, validate=True)
        except binascii.Error:
            raise Exception("MACOS_CERTIFICATE must be base64 encoded")

        certificate_password = os.getenv("MACOS_CERTIFICATE_PASSPHRASE")
        if not certificate_password:
            raise Exception(
                "MACOS_CERTIFICATE_PASSPHRASE environment variable is required"
            )

        with tempfile.TemporaryDirectory() as tempdirname:
            cert = Path(tempdirname).joinpath("certificate.p12")
            cert.write_bytes(certificate)
            import_certificate(
                path=cert, name="Developer Application", password=certificate_password
            )

    apple_certs = Path("apple-certs").resolve()
    with log_group("Import provisioning profile"):
        import_certificate(
            path=apple_certs.joinpath("artichoke-provisioning-profile-signing.cer")
        )

    with log_group("Import certificate chain"):
        import_certificate(path=apple_certs.joinpath("DeveloperIDG2CA.cer"))

    with log_group("Show codesigning identities"):
        run_command_with_merged_output(
            ["security", "find-identity", "-p", "codesigning", str(keychain_path())]
        )


def setup_codesigning_and_notarization_keychain(*, keychain_password: str) -> None:
    """
    Create and prepare a keychain for the codesigning and notarization process.

    A new keychain with the given password is created and set to be ephemeral.
    Notarization credentials and codesigning certificates are imported into the
    keychain.
    """

    create_keychain(keychain_password=keychain_password)
    import_notarization_credentials()
    import_codesigning_certificate()

    with log_group("Prepare keychain for codesigning"):
        # security set-key-partition-list \
        #   -S "apple-tool:,apple:,codesign:" \
        #   -s -k "$keychain_password" "$keychain_path"
        run_command_with_merged_output(
            [
                "security",
                "set-key-partition-list",
                "-S",
                "apple-tool:,apple:,codesign:",
                "-s",
                "-k",
                keychain_password,
                str(keychain_path()),
            ]
        )


def codesign_binary(*, binary_path: Path) -> None:
    """
    Run the codesigning process on the given binary.
    """

    # /usr/bin/codesign \
    #   --keychain "$keychain_path" \
    #   --sign "Developer ID Application: Ryan Lopopolo (VDKP67932G)" \
    #   --options runtime \
    #   --strict=all \
    #   --timestamp \
    #   --verbose \
    #   --force \
    #   "$binary_path"
    with log_group(f"Run codesigning [{binary_path.name}]"):
        run_command_with_merged_output(
            [
                "/usr/bin/codesign",
                "--keychain",
                str(keychain_path()),
                "--sign",
                codesigning_identity(),
                # Enable hardend runtime:
                #
                # - https://developer.apple.com/documentation/security/hardened_runtime
                "--options=runtime",
                "--strict=all",
                "--timestamp",
                "-vvv",
                "--force",
                str(binary_path),
            ]
        )


def setup_dmg_icon(*, dest: Path, url: str) -> None:
    """
    Fetch a .icns file from the given URL and set it as the volume icon for
    the DMG mounted at the given destination.
    """

    with log_group("Set disk image icon"):
        dmg_icns_path = dest.joinpath(".VolumeIcon.icns")
        with tempfile.TemporaryDirectory() as tempdirname:
            icns = Path(tempdirname).joinpath(".VolumeIcon.icns")

            print(f"Fetching DMG icns file at {url}")
            urllib.request.urlretrieve(url, str(icns))

            print("Copying downloaded icns file to DMG archive")
            shutil.copy(icns, dmg_icns_path)

        run_command_with_merged_output(
            ["/usr/bin/SetFile", "-c", "icnC", str(dmg_icns_path)]
        )
        # Tell the volume that it has a special file attribute
        run_command_with_merged_output(["/usr/bin/SetFile", "-a", "C", str(dest)])
        print("DMG icns file set!")


def create_notarization_bundle(
    *,
    release_name: str,
    binaries: list[Path],
    resources: list[Path],
    dmg_icon_url: Optional[str],
) -> Path:
    """
    Create a disk image with the codesigned binaries to submit to the Apple
    notarization service and prepare for distribution.

    Returns `Path` object to the newly created DMG archive.

    This method is influenced by `create-dmg`:
    https://github.com/create-dmg/create-dmg/blob/412e99352bacef0f05f9abe6cc4348a627b7ac56/create-dmg
    """

    stage = Path("dist").joinpath(release_name)
    dmg_writable = Path("dist").joinpath(f"{release_name}-temp.dmg")
    dmg = Path("dist").joinpath(f"{release_name}.dmg")

    with log_group("Create disk image for notarization"):
        dmg.unlink(missing_ok=True)
        try:
            shutil.rmtree(stage)
        except FileNotFoundError:
            pass
        os.makedirs(stage, exist_ok=True)

        for binary in binaries:
            shutil.copy(binary, stage)
        for resource in resources:
            shutil.copy(resource, stage)

        # notarytool submit works only with UDIF disk images, signed "flat"
        # installer packages, and zip files.
        #
        # Format types:
        #
        # UDRW - UDIF read/write image
        # UDZO - UDIF zlib-compressed image
        # ULFO - UDIF lzfse-compressed image (OS X 10.11+ only)
        # ULMO - UDIF lzma-compressed image (macOS 10.15+ only)

        # /usr/bin/hdiutil create \
        #    -volname "Artichoke Ruby nightly" \
        #    -srcfolder "$release_name" \
        #    -ov -format UDRW name.dmg
        run_command_with_merged_output(
            [
                "/usr/bin/hdiutil",
                "create",
                "-volname",
                disk_image_volume_name(),
                "-srcfolder",
                str(stage),
                "-ov",
                "-format",
                # Create a read/write image so we can set the DMG icon
                "UDRW",
                "-verbose",
                str(dmg_writable),
            ]
        )

    if dmg_icon_url:
        with attach_disk_image(dmg_writable, readwrite=True) as mounted_image:
            setup_dmg_icon(dest=mounted_image, url=dmg_icon_url)

    with log_group("Shrink disk image to fit"):
        run_command_with_merged_output(
            [
                "/usr/bin/hdiutil",
                "resize",
                "-size",
                f"{get_image_size(dmg_writable)}m",
                str(dmg_writable),
            ]
        )

    with log_group("Compress disk image"):
        run_command_with_merged_output(
            [
                "/usr/bin/hdiutil",
                "convert",
                str(dmg_writable),
                "-format",
                "UDZO",
                "-imagekey",
                "zlib-level=9",
                "-o",
                str(dmg),
            ]
        )

        dmg_writable.unlink()

    codesign_binary(binary_path=dmg)
    return dmg


def notarize_bundle(*, bundle: Path) -> None:
    """
    Submit the bundle to Apple for notarization using notarytool.

    This method will block until the notarization process is complete.

    https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution/customizing_the_notarization_workflow
    """

    notarization_request = None

    # xcrun notarytool submit "$bundle_name" \
    #   --keychain-profile "$notarytool_credentials_profile" \
    #   --keychain "$keychain_path" \
    #   --wait
    with log_group("Notarize disk image"):
        proc = subprocess.run(
            [
                "/usr/bin/xcrun",
                "notarytool",
                "submit",
                bundle,
                "--keychain-profile",
                notarytool_credentials_profile(),
                "--keychain",
                str(keychain_path()),
                "--wait",
            ],
            check=True,
            capture_output=True,
            text=True,
        )
        if proc.stderr:
            print(proc.stderr, file=sys.stderr)
        for line in proc.stdout.splitlines():
            print(line.rstrip())
            line = line.strip()
            if line.startswith("id: "):
                notarization_request = line.removeprefix("id: ")

    if not notarization_request:
        raise Exception("Notarization request did not return an id on success")

    # xcrun notarytool log \
    #   2efe2717-52ef-43a5-96dc-0797e4ca1041 \
    #  --keychain-profile "AC_PASSWORD" \
    #   developer_log.json
    with log_group("Fetch notarization logs"):
        with tempfile.TemporaryDirectory() as tempdirname:
            logs = Path(tempdirname).joinpath("notarization_logs.json")
            subprocess.run(
                [
                    "/usr/bin/xcrun",
                    "notarytool",
                    "log",
                    notarization_request,
                    "--keychain-profile",
                    notarytool_credentials_profile(),
                    "--keychain",
                    str(keychain_path()),
                    str(logs),
                ],
                check=True,
            )
            with logs.open("r") as log:
                log_json = json.load(log)
                print(json.dumps(log_json, indent=4))


def staple_bundle(*, bundle: Path) -> None:
    """
    Staple the diskimage with `stapler`.
    """

    with log_group("Staple disk image"):
        run_command_with_merged_output(
            ["/usr/bin/xcrun", "stapler", "staple", "-v", str(bundle)]
        )


def validate(*, bundle: Path, binary_names: list[str]) -> None:
    """
    Verify the stapled disk image and codesigning of binaries within it.
    """

    with log_group("Verify disk image staple"):
        run_command_with_merged_output(
            ["/usr/bin/xcrun", "stapler", "validate", "-v", str(bundle)]
        )

    with log_group("Verify disk image signature"):
        # spctl -a -t open \
        #   --context context:primary-signature \
        #   2022-09-03-test-codesign-notarize-dmg-v1.dmg \
        #   -v
        run_command_with_merged_output(
            [
                "/usr/sbin/spctl",
                "-a",
                "-t",
                "open",
                "--context",
                "context:primary-signature",
                str(bundle),
                "-v",
            ]
        )

    with attach_disk_image(bundle) as mounted_image:
        for binary in binary_names:
            mounted_binary = mounted_image.joinpath(binary)
            with log_group(f"Verify signature: {binary}"):
                run_command_with_merged_output(
                    [
                        "/usr/bin/codesign",
                        "--verify",
                        "--check-notarization",
                        "--deep",
                        "--strict=all",
                        "-vvv",
                        str(mounted_binary),
                    ]
                )

            with log_group(f"Display signature: {binary}"):
                run_command_with_merged_output(
                    [
                        "/usr/bin/codesign",
                        "--display",
                        "--check-notarization",
                        "-vvv",
                        str(mounted_binary),
                    ]
                )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Create Apple code signatures and notarized archives"
    )
    parser.add_argument(
        "-b",
        "--binary",
        action="append",
        dest="binaries",
        required=True,
        type=Path,
        help="path to binary to codesign",
    )
    parser.add_argument(
        "-r",
        "--resource",
        action="append",
        dest="resources",
        required=True,
        type=Path,
        help="path to resource to include in archive",
    )

    parser.add_argument(
        "-v",
        "--version",
        action="version",
        version=f"%(prog)s {MACOS_SIGN_AND_NOTARIZE_VERSION}",
    )
    parser.add_argument("release", help="release name")
    args = parser.parse_args()

    for binary in args.binaries:
        if not binary.is_file():
            print(f"Error: binary file {binary} does not exist", file=sys.stderr)
            return 1
    for resource in args.resources:
        if not resource.is_file():
            print(f"Error: resource file {resource} does not exist", file=sys.stderr)
            return 1

    dmg_icon_url = None
    if args.dmg_icon_url:
        if len(args.dmg_icon_url) > 1:
            print(
                (
                    "Error: Too many DMG icon URLs provided. "
                    "At most one DMG icon URL may be provided."
                ),
                file=sys.stderr,
            )
            return 1
        dmg_icon_url = args.dmg_icon_url[0]

    try:
        emit_metadata()

        keychain_password = secrets.token_urlsafe()
        setup_codesigning_and_notarization_keychain(keychain_password=keychain_password)

        for binary in args.binaries:
            codesign_binary(binary_path=binary)

        bundle = create_notarization_bundle(
            release_name=args.release,
            binaries=args.binaries,
            resources=args.resources,
            dmg_icon_url=dmg_icon_url,
        )
        notarize_bundle(bundle=bundle)
        staple_bundle(bundle=bundle)

        validate(bundle=bundle, binary_names=[binary.name for binary in args.binaries])
        set_output(name="asset", value=str(bundle))
        set_output(name="content_type", value="application/x-apple-diskimage")

        return 0
    except subprocess.CalledProcessError as e:
        print("Error: failed to invoke command", file=sys.stderr)
        print(f"    Command: {e.cmd}", file=sys.stderr)
        print(f"    Return Code: {e.returncode}", file=sys.stderr)
        if e.stdout:
            print()
            print("Output:", file=sys.stderr)
            for line in e.stdout.splitlines():
                print(f"    {line}", file=sys.stderr)
        if e.stderr:
            print()
            print("Error Output:", file=sys.stderr)
            for line in e.stderr.splitlines():
                print(f"    {line}", file=sys.stderr)
        print()
        print(traceback.format_exc(), file=sys.stderr)
        return e.returncode
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        print(traceback.format_exc(), file=sys.stderr)
        return 1
    finally:
        # Purge keychain.
        # TODO: Uncomment after everything's ready to run in GHA
        # delete_keychain()
        pass


if __name__ == "__main__":
    sys.exit(main())
