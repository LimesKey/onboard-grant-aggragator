# 1. This tells docker to use the Rust official image
FROM rust:latest
# 2. Copy the files in your machine to the Docker image
COPY ./ ./

# Build your program for release
RUN cargo build --release

# Run the binary
CMD ["./target/release/OnboardGrant"]
EXPOSE 8521
ENV AIRTABLE_API=get_your_own_airtable_api_key
ENV GITHUB_API=get_your_own_github_api_key