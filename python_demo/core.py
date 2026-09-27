"""Core functionality for the python_demo package.

This module contains the logic that the CLI delegates to. It demonstrates
intermediate‑level concerns such as network I/O, data validation with
pydantic, and error handling.
"""

from __future__ import annotations

import json
from typing import Any

import requests
from pydantic import BaseModel, ValidationError, Field

from .models import DemoModel


def fetch_json(url: str) -> DemoModel:
    """Fetch a JSON document from *url* and validate it against :class:`DemoModel`.

    Parameters
    ----------
    url:
        The HTTP endpoint that returns a JSON object.

    Returns
    -------
    DemoModel
        An instance populated with the validated data.

    Raises
    ------
    requests.HTTPError
        If the HTTP request fails (non‑2xx status code).
    ValidationError
        If the returned JSON does not conform to ``DemoModel``.
    json.JSONDecodeError
        If the response body is not valid JSON.
    """
    response = requests.get(url, timeout=10)
    response.raise_for_status()
    # ``response.json()`` would parse JSON but we keep the raw text to surface
    # JSONDecodeError explicitly for the caller.
    data: Any = json.loads(response.text)
    # Validate against the Pydantic model – this will raise ValidationError if
    # the structure does not match.
    return DemoModel.model_validate(data)


def example_usage() -> None:
    """Simple demonstration function used in the README.

    It fetches a known public JSON endpoint and prints the parsed model.
    """
    example_url = "https://jsonplaceholder.typicode.com/todos/1"
    try:
        model = fetch_json(example_url)
        print(model)
    except Exception as exc:
        print(f"Failed to fetch or validate data: {exc}")


if __name__ == "__main__":
    example_usage()
