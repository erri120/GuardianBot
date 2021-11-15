FROM rust:1.56-slim

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

#ENV DISCORD_TOKEN=""
#ENV APPLICATION_ID=""

ENTRYPOINT ["cargo", "run", "--release"]
