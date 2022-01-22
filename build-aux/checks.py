#!/usr/bin/env python3
# Source: https://gitlab.gnome.org/GNOME/fractal/blob/master/hooks/pre-commit.hook

import os
import subprocess
import sys
import time
from argparse import Namespace
from pathlib import Path
from typing import List, Optional, Tuple
from xml.etree import ElementTree

ERR = "\033[1;31m"
POS = "\033[32m"
NEG = "\033[31m"
ENDC = "\033[0m"

FAILED = f"{ERR}FAILED{ENDC}"
OK = f"{POS}ok{ENDC}"
ERROR = f"{NEG}error{ENDC}"


class MissingDependencyError(Exception):
    def __init__(self, whats_missing: str, install_command=None):
        self._whats_missing = whats_missing
        self._install_command = install_command

    def __str__(self):
        return f"{ERROR}: Missing dependency `{self._whats_missing}`"

    def suggestion(self) -> str:
        message = f"Please install `{self._whats_missing}` first "

        if self._install_command is not None:
            message += f"by running `{self._install_command}`"

        return message


class FailedCheckError(Exception):
    def __init__(self, error_message=None, suggestion_message=None):
        self._error_message = error_message
        self._suggestion_message = suggestion_message

    def message(self) -> Optional[str]:
        if self._error_message is not None:
            return f"{ERROR}: {self._error_message}"
        else:
            return None

    def suggestion(self) -> str:
        message = "Please fix the above issues"

        if self._suggestion_message is not None:
            message += f", {self._suggestion_message}"

        return message


class Check:
    def version(self) -> Optional[str]:
        return None

    def subject(self) -> str:
        raise NotImplementedError

    def run(self):
        raise NotImplementedError


class Rustfmt(Check):
    """Run rustfmt to enforce code style."""

    def version(self):
        try:
            return get_output(["cargo", "fmt", "--version"])
        except FileNotFoundError:
            return None

    def subject(self):
        return "code style"

    def run(self):
        try:
            return_code = run(["cargo", "fmt", "--all", "--", "--check"])
        except FileNotFoundError:
            raise MissingDependencyError(
                "cargo fmt", install_command="rustup component add rustfmt"
            )

        if return_code != 0:
            raise FailedCheckError(
                suggestion_message="either manually or by running `cargo fmt --all`"
            )


class Typos(Check):
    """Run typos to check for spelling mistakes."""

    def version(self):
        try:
            return get_output(["typos", "--version"])
        except FileNotFoundError:
            return None

    def subject(self):
        return "spelling mistakes"

    def run(self):
        try:
            return_code = run(["typos", "--color", "always"])
        except FileNotFoundError:
            raise MissingDependencyError(
                "typos", install_command="cargo install typos-cli"
            )

        if return_code != 0:
            raise FailedCheckError(
                suggestion_message="either manually or by running `typos -w`"
            )


class PotfilesAlphabetically(Check):
    """Check if files in POTFILES are sorted alphabetically.

    This assumes the following:
        - POTFILES is located at 'po/POTFILES.in'
    """

    def subject(self):
        return "po/POTFILES.in alphabetical order"

    def run(self):
        files = self._get_files()

        for file, sorted_file in zip(files, sorted(files)):
            if file != sorted_file:
                raise FailedCheckError(
                    error_message=f"Found file `{file}` before `{sorted_file}` in POTFILES.in"
                )

    @staticmethod
    def _get_files() -> List[str]:
        with open("po/POTFILES.in") as potfiles:
            return [line.strip() for line in potfiles.readlines()]


class PotfilesExist(Check):
    """Check if all files in POTFILES exist.

    This assumes the following:
        - POTFILES is located at 'po/POTFILES.in'
    """

    def subject(self):
        return "po/POTFILES.in all files exist"

    def run(self):
        files = self._get_non_existent_files()
        n_files = len(files)

        if n_files > 0:
            message = [
                f"Found {n_files} file{'s'[:n_files^1]} in POTFILES.in that does not exist:"
            ]

            for file in files:
                message.append(str(file))

            raise FailedCheckError(error_message="\n".join(message))

    @staticmethod
    def _get_non_existent_files() -> List[Path]:
        files = []

        with open("po/POTFILES.in") as potfiles:
            for line in potfiles.readlines():
                file = Path(line.strip())
                if not file.exists():
                    files.append(file)

        return files


class PotfilesSanity(Check):
    """Check if all files with translatable strings are present and only those.

    This assumes the following:
        - POTFILES is located at 'po/POTFILES.in'
        - UI (Glade) files are located in 'data/resources/ui' and use 'translatable="yes"'
        - Rust files are located in 'src' and use '*gettext' methods or macros
    """

    def subject(self):
        return "po/POTFILES.in sanity"

    def run(self):
        (ui_potfiles, rust_potfiles) = self._get_potfiles()

        ui_potfiles, ui_files = self._remove_common_files(
            ui_potfiles, self._get_ui_files()
        )
        rust_potfiles, rust_files = self._remove_common_files(
            rust_potfiles, self._get_rust_files()
        )

        n_potfiles = len(rust_potfiles) + len(ui_potfiles)
        if n_potfiles != 0:
            message = [
                f"Found {n_potfiles} file{'s'[:n_potfiles^1]} in POTFILES.in without translatable strings:"
            ]

            for file in rust_potfiles:
                message.append(str(file))

            for file in ui_potfiles:
                message.append(str(file))

            raise FailedCheckError(error_message="\n".join(message))

        n_files = len(rust_files) + len(ui_files)
        if n_files != 0:
            message = [
                f"Found {n_files} file{'s'[:n_files^1]} with translatable strings not present in POTFILES.in:"
            ]

            for file in rust_files:
                message.append(str(file))

            for file in ui_files:
                message.append(str(file))

            raise FailedCheckError(error_message="\n".join(message))

    @staticmethod
    def _remove_common_files(set_a: List[Path], set_b: List[Path]):
        for file_a in list(set_a):
            for file_b in list(set_b):
                if file_a == file_b:
                    set_a.remove(file_b)
                    set_b.remove(file_b)
        return set_a, set_b

    @staticmethod
    def _get_potfiles() -> Tuple[List[Path], List[Path]]:
        ui_potfiles = []
        rust_potfiles = []

        with open("po/POTFILES.in") as potfiles:
            for line in potfiles.readlines():
                file = Path(line.strip())

                if file.suffix == ".ui":
                    ui_potfiles.append(file)
                elif file.suffix == ".rs":
                    rust_potfiles.append(file)

        return (ui_potfiles, rust_potfiles)

    @staticmethod
    def _get_ui_files() -> List[Path]:
        output = get_output(
            "grep -lIr 'translatable=\"yes\"' data/resources/ui/*", shell=True
        )
        return list(map(lambda s: Path(s), output.splitlines()))

    @staticmethod
    def _get_rust_files() -> List[Path]:
        output = get_output(r"grep -lIrE 'gettext[!]?\(' src/*", shell=True)
        return list(map(lambda s: Path(s), output.splitlines()))


class Resources(Check):
    """Check if files in data/resources/resources.gresource.xml are sorted alphabetically.

    This assumes the following:
        - gresource file is located in `data/resources/resources.gresource.xml`
        - only one gresource in the file
    """

    def subject(self):
        return "data/resources/resources.gresource.xml"

    def run(self):
        # Do not consider path suffix on sorting
        class File:
            def __init__(self, path: str):
                self._path = Path(path)

            def __str__(self):
                return self._path.__str__()

            def __lt__(self, other):
                return self._path.with_suffix("") < other._path.with_suffix("")

        tree = ElementTree.parse("data/resources/resources.gresource.xml")
        gresource = tree.find("gresource")
        files = [File(element.text) for element in gresource.findall("file")]
        sorted_files = sorted(files)

        for file, sorted_file in zip(files, sorted_files):
            if file != sorted_file:
                raise FailedCheckError(
                    error_message=f"Found file `{file}` before `{sorted_file}` in resources.gresource.xml"
                )


class Runner:

    _checks: List[Check] = []

    def add(self, check: Check):
        self._checks.append(check)

    def run_all(self) -> bool:
        """Returns true if all are successful"""

        n_checks = len(self._checks)

        print(f"running {n_checks} checks")

        start_time = time.time()
        successful_checks = []

        for check in self._checks:
            try:
                check.run()
            except FailedCheckError as e:
                remark = FAILED

                if e.message() is not None:
                    print("")
                    print(e.message())

                print("")
                print(e.suggestion())
                print("")
            except MissingDependencyError as e:
                remark = FAILED
                print("")
                print(e)
                print("")
                print(e.suggestion())
                print("")
            else:
                remark = OK
                successful_checks.append(check)

            self._print_check(
                check.subject(), check.version() if args.verbose else None, remark
            )

        check_duration = time.time() - start_time
        n_successful_checks = len(successful_checks)

        print("")
        self._print_result(n_checks, n_successful_checks, check_duration)

        return n_successful_checks == n_checks

    @staticmethod
    def _print_result(total: int, n_successful: int, duration: float):
        n_failed = total - n_successful

        if total == n_successful:
            result = OK
        else:
            result = FAILED

        print(
            f"test result: {result}. {n_successful} passed; {n_failed} failed; finished in {duration:.2f}s"
        )

    @staticmethod
    def _print_check(subject: str, version: Optional[str], remark: str):
        messages = ["check", subject]

        if version is not None:
            messages.append(f"({version})")

        messages.append("...")
        messages.append(remark)

        print(" ".join(messages))


def run(args: List[str], **kwargs) -> int:
    process = subprocess.run(args, **kwargs)
    return process.returncode


def get_output(*args, **kwargs) -> str:
    process = subprocess.run(*args, capture_output=True, **kwargs)
    return process.stdout.decode("utf-8").strip()


def main(args: Namespace) -> int:
    runner = Runner()

    if not args.skip_rustfmt:
        runner.add(Rustfmt())

    if not args.skip_typos:
        runner.add(Typos())

    runner.add(PotfilesExist())
    runner.add(PotfilesSanity())
    runner.add(PotfilesAlphabetically())
    runner.add(Resources())

    if runner.run_all():
        return os.EX_OK
    else:
        return 1


if __name__ == "__main__":
    from argparse import ArgumentParser

    parser = ArgumentParser(
        description="Run conformity checks on the current Rust project"
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="Use verbose output"
    )
    parser.add_argument(
        "-sr",
        "--skip-rustfmt",
        action="store_true",
        help="Whether to skip running cargo fmt",
    )
    parser.add_argument(
        "-st", "--skip-typos", action="store_true", help="Whether to skip running typos"
    )
    args = parser.parse_args()

    sys.exit(main(args))
