#!/usr/bin/env node

/**
 * Simple Test Tool - No external dependencies
 * 
 * This tool demonstrates basic Node.js functionality without requiring npm packages
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

        // Simple echo test
        const result = {
            success: true,
            message: "Hello from Node.js!",
            received_params: params,
            node_version: process.version,
            platform: process.platform,
            timestamp: new Date().toISOString(),
            execution_id: context.execution_id || null
        };

        console.log(JSON.stringify(result, null, 2));
        process.exit(0);

    } catch (error) {
        const errorOutput = {
            success: false,
            error: error.message,
            timestamp: new Date().toISOString()
        };

        console.log(JSON.stringify(errorOutput, null, 2));
        process.exit(1);
    }
});