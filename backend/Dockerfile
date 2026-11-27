FROM python:3.12-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    GRP_PORT=8080

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY data/ /app/data/
COPY backend/server.py /app/backend/server.py

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python3 -c "import os, urllib.request; port=os.getenv('PORT', os.getenv('GRP_PORT', '8080')); urllib.request.urlopen('http://127.0.0.1:' + port + '/healthz', timeout=3).read()"

CMD ["python3", "/app/backend/server.py"]
