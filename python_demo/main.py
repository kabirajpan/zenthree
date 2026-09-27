"""Entry point for the python_demo package.

Provides a simple command-line interface demonstrating typical intermediate
project features such as argument parsing, logging and delegating to core
logic.
"""

import argparse
import logging
import sys
from .core import fetch_json


def configure_logging(verbose: bool) -> None:
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        stream=sys.stdout,
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python-demo",
        description="Demo CLI that fetches JSON from a URL and validates it.",
    )
    parser.add_argument(
        "url",
        help="The URL to fetch JSON from. Must return a JSON object compatible with the DemoModel.",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Enable verbose (debug) logging output.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    configure_logging(args.verbose)
    logger = logging.getLogger(__name__)

    try:
        data = fetch_json(args.url)
        logger.info("Successfully fetched and validated data: %s", data)
        return 0
    except Exception as exc:  # pragma: no cover – CLI entry point
        logger.error("Error: %s", exc)
        return 1


if __name__ == "__main__":
    sys.exit(main())
