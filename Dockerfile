FROM rust:latest as build
WORKDIR /usr/src/whereami
COPY . .
RUN cargo install --path .

FROM debian
COPY --from=build /usr/local/cargo/bin/whereami /usr/local/bin/whereami
CMD ["whereami"]
