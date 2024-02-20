FROM cgr.dev/chainguard/static
COPY --chown=nonroot:nonroot ./release/controller /app/
EXPOSE 8080
ENTRYPOINT ["/app/controller"]