FROM cgr.dev/chainguard/static
COPY --chown=nonroot:nonroot ./x86_64-unknown-linux-musl/release/controller /app/
EXPOSE 8080
ENTRYPOINT ["/app/controller"]