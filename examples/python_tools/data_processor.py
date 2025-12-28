#!/usr/bin/env python3
"""
Data processing tool for the workflow toolkit.
This script demonstrates more complex data processing capabilities.
"""

import json
import sys
from typing import Dict, Any, List, Union
import statistics
import re


def process_numbers(data: List[Union[int, float]], operation: str) -> Dict[str, Any]:
    """Process a list of numbers with various statistical operations."""
    if not data:
        raise ValueError("Data list cannot be empty")
    
    if not all(isinstance(x, (int, float)) for x in data):
        raise ValueError("All data items must be numbers")
    
    operations = {
        "sum": sum,
        "mean": statistics.mean,
        "median": statistics.median,
        "mode": statistics.mode,
        "min": min,
        "max": max,
        "stdev": statistics.stdev if len(data) > 1 else lambda x: 0,
        "variance": statistics.variance if len(data) > 1 else lambda x: 0,
    }
    
    if operation not in operations:
        raise ValueError(f"Unknown operation: {operation}. Supported: {list(operations.keys())}")
    
    try:
        result = operations[operation](data)
        return {
            "result": result,
            "operation": operation,
            "data_count": len(data),
            "data_range": {"min": min(data), "max": max(data)},
        }
    except statistics.StatisticsError as e:
        raise ValueError(f"Statistics error: {e}")


def process_text(text: str, operation: str) -> Dict[str, Any]:
    """Process text with various string operations."""
    if not isinstance(text, str):
        raise ValueError("Text must be a string")
    
    operations = {
        "word_count": lambda t: len(t.split()),
        "char_count": len,
        "line_count": lambda t: len(t.splitlines()),
        "uppercase": str.upper,
        "lowercase": str.lower,
        "title_case": str.title,
        "reverse": lambda t: t[::-1],
        "remove_whitespace": lambda t: re.sub(r'\s+', '', t),
        "extract_emails": lambda t: re.findall(r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b', t),
        "extract_urls": lambda t: re.findall(r'http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+', t),
    }
    
    if operation not in operations:
        raise ValueError(f"Unknown operation: {operation}. Supported: {list(operations.keys())}")
    
    result = operations[operation](text)
    
    return {
        "result": result,
        "operation": operation,
        "original_length": len(text),
        "original_word_count": len(text.split()),
    }


def process_list(data: List[Any], operation: str) -> Dict[str, Any]:
    """Process a list with various operations."""
    if not isinstance(data, list):
        raise ValueError("Data must be a list")
    
    operations = {
        "length": len,
        "unique": lambda d: list(set(d)),
        "sort": sorted,
        "reverse": lambda d: list(reversed(d)),
        "filter_numbers": lambda d: [x for x in d if isinstance(x, (int, float))],
        "filter_strings": lambda d: [x for x in d if isinstance(x, str)],
        "flatten": lambda d: [item for sublist in d for item in (sublist if isinstance(sublist, list) else [sublist])],
    }
    
    if operation not in operations:
        raise ValueError(f"Unknown operation: {operation}. Supported: {list(operations.keys())}")
    
    try:
        result = operations[operation](data)
        return {
            "result": result,
            "operation": operation,
            "original_length": len(data),
            "result_type": type(result).__name__,
        }
    except Exception as e:
        raise ValueError(f"List processing error: {e}")


def main():
    """Main entry point for the data processor tool."""
    try:
        # Read input from stdin
        input_data = sys.stdin.read()
        if not input_data.strip():
            raise ValueError("No input data provided")
        
        # Parse JSON input
        data = json.loads(input_data)
        params = data.get("params", {})
        context = data.get("context", {})
        
        # Extract processing type and operation
        process_type = params.get("type")
        operation = params.get("operation")
        input_data = params.get("data")
        
        if not process_type:
            raise ValueError("Processing type parameter is required")
        
        if not operation:
            raise ValueError("Operation parameter is required")
        
        if input_data is None:
            raise ValueError("Data parameter is required")
        
        # Process based on type
        if process_type == "numbers":
            result_data = process_numbers(input_data, operation)
        elif process_type == "text":
            result_data = process_text(input_data, operation)
        elif process_type == "list":
            result_data = process_list(input_data, operation)
        else:
            raise ValueError(f"Unknown processing type: {process_type}. Supported: numbers, text, list")
        
        # Prepare output
        output = {
            "success": True,
            "type": process_type,
            **result_data,
            "execution_id": context.get("execution_id"),
            "timestamp": context.get("started_at"),
        }
        
        # Write result to stdout as JSON
        print(json.dumps(output))
        
    except Exception as e:
        # Write error to stdout as JSON
        error_output = {
            "error": str(e),
            "error_type": type(e).__name__,
            "success": False,
        }
        print(json.dumps(error_output))
        sys.exit(1)


if __name__ == "__main__":
    main()