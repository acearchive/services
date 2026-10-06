FROM lukemathwalker/cargo-chef:latest-rust-1-slim AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --bins && mkdir /scratch

# Don't mix up `gcr.io` (Google) and `ghcr.io` (GitHub).
FROM gcr.io/distroless/cc:nonroot AS runtime

COPY --from=builder /app/target/release/acearchive-data-connector /usr/local/bin/
COPY --from=builder /app/target/release/healthcheck /usr/local/bin/
COPY --from=builder --chown=nonroot:nonroot /scratch /scratch

ENV PORT=8080
ENV SCRATCH_DIR=/scratch

EXPOSE 8080
USER nonroot

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD ["/usr/local/bin/healthcheck"]

ENTRYPOINT ["/usr/local/bin/acearchive-data-connector"]
