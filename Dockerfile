# Stage 1: Build the Rust engine
FROM rustlang/rust:nightly-slim AS builder
WORKDIR /usr/src/iglo
# Copy the Iglo source code and build it
COPY . .
RUN cargo build --release

# Stage 2: Prepare the bot runtime (Python 3.11)
FROM python:3.11-slim
# Pass Lichess bot token at build/run time
ARG LICHESS_BOT_TOKEN
ENV LICHESS_BOT_TOKEN=${LICHESS_BOT_TOKEN}

# Create directory for engine and copy binary from builder
WORKDIR /usr/src/app
RUN mkdir -p engines
COPY --from=builder /usr/src/iglo/target/release/iglo engines/iglo

# Install lichess-bot: clone repo, install requirements
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
RUN git clone https://github.com/lichess-bot-devs/lichess-bot.git \
 && cd lichess-bot \
 && python3 -m pip install --upgrade pip \
 && python3 -m pip install -r requirements.txt \
 && python3 -m pip install .

# Copy the bot configuration (see sample below)
COPY config.yml ./

# Default command: run lichess-bot (expects config.yml present)
ENTRYPOINT ["lichess-bot"]
