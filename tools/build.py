from argparse import ArgumentParser
from pathlib import Path
import subprocess
import sys


_THIS_DIR = Path(__file__).parent
_PROJECT_DIR = _THIS_DIR / ".."


def _parse_args():
    parser = ArgumentParser()
    parser.add_argument(
        "-r",
        "--run",
        action="store_true",
        default=False,
        help="If set, run the executable after build (only for non-web)",
    )
    parser.add_argument(
        "-w",
        "--web",
        action="store_true",
        default=False,
        help="If set, build for wasm32 target",
    )
    parser.add_argument(
        "-e",
        "--example",
        type=str,
        default=None,
        help="Example name to be built (if all - build all examples, if not set - build the library)",
    )
    return parser.parse_args()


def build(run: bool, web: bool, example: str):
    if run and web:
        print("ERROR: `run` and `web` can't be set simultaneously", file=sys.stderr)
        sys.exit(1)

    if run and example is None:
        print("ERROR: `Pass the `example` name to run the executable", file=sys.stderr)
        sys.exit(1)

    if run and example == "all":
        print(
            "ERROR: Can't `run` if `example` is set to 'all'. Pass a specific example name if you want to run the executable",
            file=sys.stderr,
        )
        sys.exit(1)

    cmd = ["cargo", "run" if run else "build", "--release"]

    target_args = []
    if web:
        target_args = ["--target", "wasm32-unknown-emscripten"]

    example_args = []
    if example == "all":
        example_args = ["--examples"]
    elif example is not None:
        example_args = ["--example", example]

    cmd += target_args + example_args

    print(" ".join(cmd))
    subprocess.check_call(cmd, cwd=str(_PROJECT_DIR))


if __name__ == "__main__":
    args = _parse_args()
    build(args.run, args.web, args.example)
