"""Pydantic models for the python_demo package.

The DemoModel represents a simple JSON structure used by the example
application. It demonstrates basic validation, default values, and field
metadata.
"""

from __future__ import annotations

from pydantic import BaseModel, Field


class DemoModel(BaseModel):
    """A minimal model representing a to‑do item.

    The fields correspond to the JSON returned by the example endpoint
    ``https://jsonplaceholder.typicode.com/todos/1``.
    """

    userId: int = Field(..., description="Identifier of the user that owns the todo.")
    id: int = Field(..., description="Unique identifier for the todo item.")
    title: str = Field(..., description="Short description of the todo.")
    completed: bool = Field(False, description="Whether the todo has been completed.")
