// JavaScript source for a text processing WASM module
// This can be compiled to WASM using tools like Javy or similar

function process_text() {
    // Read input from extism
    const input = Host.inputString();
    
    try {
        const data = JSON.parse(input);
        const operation = data.operation || "uppercase";
        const text = data.text || "";
        
        let result;
        switch (operation) {
            case "uppercase":
                result = text.toUpperCase();
                break;
            case "lowercase":
                result = text.toLowerCase();
                break;
            case "reverse":
                result = text.split("").reverse().join("");
                break;
            case "length":
                result = text.length;
                break;
            default:
                result = text;
        }
        
        const output = {
            result: result,
            operation: operation,
            original_length: text.length,
            processed_length: typeof result === "string" ? result.length : 0,
            status: "success"
        };
        
        Host.outputString(JSON.stringify(output));
    } catch (error) {
        const errorOutput = {
            error: error.message,
            status: "error"
        };
        Host.outputString(JSON.stringify(errorOutput));
    }
}

function transform_data() {
    const input = Host.inputString();
    
    try {
        const data = JSON.parse(input);
        const transformation = data.transformation || "identity";
        const values = data.values || [];
        
        let result;
        switch (transformation) {
            case "double":
                result = values.map(x => x * 2);
                break;
            case "square":
                result = values.map(x => x * x);
                break;
            case "sum":
                result = values.reduce((a, b) => a + b, 0);
                break;
            case "average":
                result = values.length > 0 ? values.reduce((a, b) => a + b, 0) / values.length : 0;
                break;
            default:
                result = values;
        }
        
        const output = {
            result: result,
            transformation: transformation,
            input_count: values.length,
            status: "success"
        };
        
        Host.outputString(JSON.stringify(output));
    } catch (error) {
        const errorOutput = {
            error: error.message,
            status: "error"
        };
        Host.outputString(JSON.stringify(errorOutput));
    }
}

// Export functions for extism
module.exports = {
    process_text,
    transform_data
};