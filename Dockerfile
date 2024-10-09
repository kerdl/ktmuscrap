FROM rust:1.81

ARG cwd=/ktmuscrap
ENV TZ="Europe/Moscow"

WORKDIR ${cwd}
COPY . ${cwd}

RUN cargo build --release

EXPOSE 8080
CMD ["cargo", "run", "--release"]
