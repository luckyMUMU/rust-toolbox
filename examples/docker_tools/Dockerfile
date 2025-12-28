# Simple Docker tool example
FROM alpine:latest

# Install basic tools
RUN apk add --no-cache \
    bash \
    jq \
    curl

# Create a working directory
WORKDIR /app

# Copy the tool script
COPY simple_processor.sh /app/

# Make the script executable
RUN chmod +x /app/simple_processor.sh

# Set the default command
CMD ["/app/simple_processor.sh"]