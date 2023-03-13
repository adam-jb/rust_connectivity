

# Use Rust official image as the base image
FROM rust:1.67 as build

# Set the working directory to /app
WORKDIR /app

# Copy the Rust project files to the container
COPY . .

# Build the project with Cargo
RUN cargo build --release

RUN ls -la /app/target/release/


# Create a new image with only the built executable
FROM ubuntu:20.04

# Install OpenSSL libraries
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

# Set the working directory to /app
WORKDIR /app

# Copy the built executable from the previous image
COPY --from=build /app/target/release/rust_connectivity .

# Copy src files from previous image
COPY --from=build /app/src/* src/

# Copy serialised data to be used by app
COPY --from=build /app/serialised_data/* serialised_data/

# Set calls to API running in background
RUN chmod +x ./app/call_api_each_year.sh
RUN ./app/call_api_each_year.sh &

# Set the command to run the Actix Web server
CMD ["./rust_connectivity"]
