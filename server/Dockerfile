FROM debian:stable-slim

RUN apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends --assume-yes \
        ca-certificates \
        librust-backtrace+serde-dev \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*;

WORKDIR /opt

COPY ./xindeler-updater-server /opt/xindeler-updater-server

ENV RUST_BACKTRACE=full
EXPOSE 8000
CMD [ "/opt/xindeler-updater-server" ]
