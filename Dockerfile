FROM rust:latest AS builder
RUN update-ca-certificates
ENV USER=dockeruser
ENV UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"
WORKDIR /app
COPY ./ .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
WORKDIR /app
COPY --from=builder /app/target/release/rekapundi ./
USER dockeruser:dockeruser
EXPOSE 8080
CMD ["/app/rekapundi"]
