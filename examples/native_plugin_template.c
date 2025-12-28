/*
 * Template for implementing a native plugin in C
 * 
 * This file shows the required API functions that a native plugin
 * dynamic library must implement to be compatible with the workflow toolkit.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Tool descriptor structure (must match Rust definition)
typedef struct ToolDescriptor {
    const char* name;
    const char* version;
    const char* description;
    const char* parameters_schema;
    const char* return_schema;
    const struct ToolDescriptor* next;
} ToolDescriptor;

// Plugin state structure (example)
typedef struct PluginState {
    int initialized;
    char* config_json;
    // Add your plugin-specific state here
} PluginState;

// Static tool descriptors
static ToolDescriptor example_tool = {
    .name = "example_tool",
    .version = "1.0.0",
    .description = "An example tool from native plugin",
    .parameters_schema = "{\"type\":\"object\",\"properties\":{\"input\":{\"type\":\"string\"}}}",
    .return_schema = "{\"type\":\"object\",\"properties\":{\"output\":{\"type\":\"string\"}}}",
    .next = NULL
};

// Required API functions

/**
 * Get plugin information
 * Returns: JSON string with plugin metadata
 */
const char* plugin_info() {
    return "{"
           "\"name\":\"example-native-plugin\","
           "\"version\":\"1.0.0\","
           "\"description\":\"Example native plugin\","
           "\"author\":\"Example Author\""
           "}";
}

/**
 * Initialize the plugin
 * config_json: JSON configuration string
 * Returns: Plugin handle (opaque pointer)
 */
void* plugin_init(const char* config_json) {
    PluginState* state = malloc(sizeof(PluginState));
    if (!state) {
        return NULL;
    }
    
    state->initialized = 1;
    state->config_json = strdup(config_json);
    
    printf("Native plugin initialized with config: %s\n", config_json);
    
    return state;
}

/**
 * Get list of tools provided by this plugin
 * plugin_handle: Plugin handle from plugin_init
 * Returns: Linked list of tool descriptors
 */
const ToolDescriptor* plugin_get_tools(void* plugin_handle) {
    PluginState* state = (PluginState*)plugin_handle;
    if (!state || !state->initialized) {
        return NULL;
    }
    
    // Return the static tool descriptor
    return &example_tool;
}

/**
 * Execute a tool
 * plugin_handle: Plugin handle from plugin_init
 * tool_name: Name of the tool to execute
 * params_json: JSON parameters for the tool
 * context_json: JSON execution context
 * Returns: JSON result string
 */
const char* plugin_execute_tool(void* plugin_handle, 
                                const char* tool_name,
                                const char* params_json, 
                                const char* context_json) {
    PluginState* state = (PluginState*)plugin_handle;
    if (!state || !state->initialized) {
        return "{\"error\":\"Plugin not initialized\"}";
    }
    
    printf("Executing tool: %s\n", tool_name);
    printf("Parameters: %s\n", params_json);
    printf("Context: %s\n", context_json);
    
    if (strcmp(tool_name, "example_tool") == 0) {
        // Simple example: echo the input
        static char result[1024];
        snprintf(result, sizeof(result), 
                "{\"output\":\"Hello from native plugin! Input was: %s\"}", 
                params_json);
        return result;
    }
    
    return "{\"error\":\"Unknown tool\"}";
}

/**
 * Shutdown the plugin
 * plugin_handle: Plugin handle from plugin_init
 */
void plugin_shutdown(void* plugin_handle) {
    PluginState* state = (PluginState*)plugin_handle;
    if (state) {
        printf("Shutting down native plugin\n");
        
        if (state->config_json) {
            free(state->config_json);
        }
        
        state->initialized = 0;
        free(state);
    }
}

/*
 * To build this as a shared library:
 * 
 * gcc -shared -fPIC -o libexample_plugin.so native_plugin_template.c
 * 
 * Then you can use it with the workflow toolkit by specifying the path
 * to libexample_plugin.so in your plugin configuration.
 */