FROM rust:1.79-slim-bookworm

# Instalação das dependências nativas para fiscal_real e fiscal_xmldsig_real
RUN apt-get update && apt-get install -y \
    pkg-config \
    build-essential \
    libssl-dev \
    openssl \
    perl \
    libxml2 \
    libxml2-dev \
    libxmlsec1 \
    libxmlsec1-dev \
    xmlsec1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# O código será montado no docker-compose como volume
# Este contêiner serve apenas para rodar o build/check
CMD ["cargo", "check", "-p", "aureon-api-local"]
