#!/usr/bin/env python3
"""
Simple calculator tool for the workflow toolkit.
This script demonstrates how to create a Python tool that can be used with the plugin system.
"""

import json
import sys
from typing import Dict, Any, Union


def add(a: Union[int, float], b: Union[int, float]) -> Union[int, float]:
    """Add two numbers."""
    return a + b


def subtract(a: Union[int, float], b: Union[int, float]) -> Union[int, float]:
    """Subtract two numbers."""
    return a - b


def multiply(a: Union[int, float], b: Union[int, float]) -> Union[int, float]:
    """Multiply two numbers."""
    return a * b


def divide(a: Union[int, float], b: Union[int, float]) -> Union[int, float]:
    """Divide two numbers."""
    if b == 0:
        raise ValueError("Division by zero is not allowed")
    return a / b


def power(a: Union[int, float], b: Union[int, float]) -> Union[int, float]:
    """Raise a to the power of b."""
    return a ** b


def main():
    """Main entry point for the calculator tool."""
    try:
        # Read input from stdin
        input_data = sys.stdin.read()
        if not input_data.strip():
            raise ValueError("No input data provided")
        
        # Parse JSON input
        data = json.loads(input_data)
        params = data.get("params", {})
        context = data.get("context", {})
        
        # Extract operation and operands
        operation = params.get("operation")
        if not operation:
            raise ValueError("Operation parameter is required")
        
        a = params.get("a")
        b = params.get("b")
        
        if a is None or b is None:
            raise ValueError("Both 'a' and 'b' parameters are required")
        
        # Validate operands are numbers
        if not isinstance(a, (int, float)) or not isinstance(b, (int, float)):
            raise ValueError("Operands must be numbers")
        
        # Perform the operation
        operations = {
            "add": add,
            "subtract": subtract,
            "multiply": multiply,
            "divide": divide,
            "power": power,
        }
        
        if operation not in operations:
            raise ValueError(f"Unknown operation: {operation}. Supported operations: {list(operations.keys())}")
        
        result = operations[operation](a, b)
        
        # Prepare output
        output = {
            "result": result,
            "operation": operation,
            "operands": {"a": a, "b": b},
            "execution_id": context.get("execution_id"),
            "timestamp": context.get("started_at"),
        }
        
        # Write result to stdout as JSON
        print(json.dumps(output))
        
    except Exception as e:
        # Write error to stdout as JSON (not stderr, so the plugin can parse it)
        error_output = {
            "error": str(e),
            "error_type": type(e).__name__,
            "success": False,
        }
        print(json.dumps(error_output))
        sys.exit(1)


if __name__ == "__main__":
    main()