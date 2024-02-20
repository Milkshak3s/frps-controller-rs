FROM cgr.dev/chainguard/static
COPY --chown=nonroot:nonroot ./target/x86_64-unknown-linux-musl/controller /app/
EXPOSE 8080
ENTRYPOINT ["/app/controller"]