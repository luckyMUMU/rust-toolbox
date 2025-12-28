#!/usr/bin/env node

/**
 * Data Processor Tool for Node.js Plugin Example
 * 
 * This tool demonstrates data processing operations using lodash and shows how to:
 * - Process arrays and objects
 * - Use external npm dependencies
 * - Handle complex data transformations
 * - Return structured results
 */

const _ = require('lodash');
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

        if (!params.data) {
            throw new Error('Missing required parameter: data');
        }

        const { operation, data, options = {} } = params;
        let result;

        // Perform data processing based on operation
        switch (operation.toLowerCase()) {
            case 'filter':
                if (!options.predicate) {
                    throw new Error('Filter operation requires predicate in options');
                }
                // Simple predicate evaluation (for demo purposes)
                if (options.predicate.type === 'greater_than') {
                    result = _.filter(data, item => item > options.predicate.value);
                } else if (options.predicate.type === 'has_property') {
                    result = _.filter(data, item => _.has(item, options.predicate.property));
                } else {
                    throw new Error(`Unsupported predicate type: ${options.predicate.type}`);
                }
                break;

            case 'map':
                if (!options.transform) {
                    throw new Error('Map operation requires transform in options');
                }
                if (options.transform.type === 'multiply') {
                    result = _.map(data, item => item * options.transform.value);
                } else if (options.transform.type === 'extract_property') {
                    result = _.map(data, item => _.get(item, options.transform.property));
                } else {
                    throw new Error(`Unsupported transform type: ${options.transform.type}`);
                }
                break;

            case 'group_by':
                if (!options.key) {
                    throw new Error('Group by operation requires key in options');
                }
                result = _.groupBy(data, options.key);
                break;

            case 'sort':
                const sortKey = options.key || null;
                const sortOrder = options.order || 'asc';
                if (sortKey) {
                    result = _.orderBy(data, [sortKey], [sortOrder]);
                } else {
                    result = _.orderBy(data, [], [sortOrder]);
                }
                break;

            case 'unique':
                const uniqueKey = options.key || null;
                if (uniqueKey) {
                    result = _.uniqBy(data, uniqueKey);
                } else {
                    result = _.uniq(data);
                }
                break;

            case 'aggregate':
                if (!options.aggregation) {
                    throw new Error('Aggregate operation requires aggregation in options');
                }
                const { aggregation } = options;
                switch (aggregation.type) {
                    case 'sum':
                        result = _.sumBy(data, aggregation.key || (x => x));
                        break;
                    case 'count':
                        result = data.length;
                        break;
                    case 'average':
                        result = _.meanBy(data, aggregation.key || (x => x));
                        break;
                    case 'min':
                        result = _.minBy(data, aggregation.key || (x => x));
                        break;
                    case 'max':
                        result = _.maxBy(data, aggregation.key || (x => x));
                        break;
                    default:
                        throw new Error(`Unsupported aggregation type: ${aggregation.type}`);
                }
                break;

            case 'transform':
                // Complex transformation using lodash
                result = _.chain(data)
                    .filter(item => item !== null && item !== undefined)
                    .map(item => {
                        if (typeof item === 'object') {
                            return _.mapValues(item, value => 
                                typeof value === 'string' ? value.trim() : value
                            );
                        }
                        return item;
                    })
                    .value();
                break;

            default:
                throw new Error(`Unsupported operation: ${operation}. Supported operations: filter, map, group_by, sort, unique, aggregate, transform`);
        }

        // Return success result
        const output = {
            success: true,
            result: result,
            operation: operation,
            input_count: Array.isArray(data) ? data.length : (typeof data === 'object' ? Object.keys(data).length : 1),
            output_count: Array.isArray(result) ? result.length : (typeof result === 'object' ? Object.keys(result).length : 1),
            timestamp: new Date().toISOString(),
            execution_id: context.execution_id || null,
            lodash_version: _.VERSION
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