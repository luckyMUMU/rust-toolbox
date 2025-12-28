#!/usr/bin/env node

/**
 * Simple Calculator Tool for Node.js Plugin Example
 * 
 * This tool demonstrates basic arithmetic operations and shows how to:
 * - Read input from stdin as JSON
 * - Validate parameters
 * - Perform calculations
 * - Return results as JSON
 * - Handle errors gracefully
 */

const readline = require('readline');

// Create readline interface for reading from stdin
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

// Read all input from stdin
let inputData = '';
rl.on('line', (line) => {
    inputData += line + '\n';
});

rl.on('close', () => {
    try {
        // Parse input JSON
        const input = JSON.parse(inputData.trim());
        const params = input.params || {};
        const context = input.context || {};

        // Log context information for debugging
        if (context.execution_id) {
            console.error(`[DEBUG] Execution ID: ${context.execution_id}`);
        }

        // Validate required parameters
        if (!params.operation) {
            throw new Error('Missing required parameter: operation');
        }

        if (typeof params.a !== 'number' || typeof params.b !== 'number') {
            throw new Error('Parameters a and b must be numbers');
        }

        const { operation, a, b } = params;
        let result;

        // Perform calculation based on operation
        switch (operation.toLowerCase()) {
            case 'add':
            case '+':
                result = a + b;
                break;
            case 'subtract':
            case '-':
                result = a - b;
                break;
            case 'multiply':
            case '*':
                result = a * b;
                break;
            case 'divide':
            case '/':
                if (b === 0) {
                    throw new Error('Division by zero is not allowed');
                }
                result = a / b;
                break;
            case 'power':
            case '**':
                result = Math.pow(a, b);
                break;
            case 'modulo':
            case '%':
                if (b === 0) {
                    throw new Error('Modulo by zero is not allowed');
                }
                result = a % b;
                break;
            default:
                throw new Error(`Unsupported operation: ${operation}. Supported operations: add, subtract, multiply, divide, power, modulo`);
        }

        // Return success result
        const output = {
            success: true,
            result: result,
            operation: operation,
            operands: { a, b },
            timestamp: new Date().toISOString(),
            execution_id: context.execution_id || null
        };

        console.log(JSON.stringify(output, null, 2));
        process.exit(0);

    } catch (error) {
        // Return error result
        const errorOutput = {
            success: false,
            error: error.message,
            timestamp: new Date().toISOString(),
            execution_id: (inputData && JSON.parse(inputData).context?.execution_id) || null
        };

        console.log(JSON.stringify(errorOutput, null, 2));
        process.exit(1);
    }
});

// Handle process termination
process.on('SIGINT', () => {
    const errorOutput = {
        success: false,
        error: 'Process interrupted',
        timestamp: new Date().toISOString()
    };
    console.log(JSON.stringify(errorOutput, null, 2));
    process.exit(1);
});

process.on('SIGTERM', () => {
    const errorOutput = {
        success: false,
        error: 'Process terminated',
        timestamp: new Date().toISOString()
    };
    console.log(JSON.stringify(errorOutput, null, 2));
    process.exit(1);
});