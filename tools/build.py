from argparse import ArgumentParser
from pathlib import Path
import subprocess


_THIS_DIR = Path(__file__).parent
_PROJECT_DIR = _THIS_DIR / ".."


def _parse_args():
    parser = ArgumentParser()
    parser.add_argument(
        "--web",
        action="store_true",
        default=False,
        help="If set, build for wasm32 target",
    )
    parser.add_argument(
        "--example",
        type=str,
        default=None,
        help="Example name to be built (if all - build all examples, if not set - build the library)",
    )
    return parser.parse_args()


def build(web: bool, example: str):
    cmd = ["cargo", "build", "--release"]

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
    build(args.web, args.example)
