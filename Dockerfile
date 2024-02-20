FROM cgr.dev/chainguard/static
COPY --chown=nonroot:nonroot ./target/release/controller /app/
EXPOSE 8080
ENTRYPOINT ["/app/controller"]