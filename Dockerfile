FROM rust:1.72.1 as build

# create a new empty project
RUN USER=root cargo new --bin caseus-foratus-helveticus
WORKDIR /caseus-foratus-helveticus

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/caseus_foratus_helveticus*
RUN cargo build --release

# our final base
FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl ca-certificates

# copy the build artifact from the build stage
COPY --from=build /caseus-foratus-helveticus/target/release/caseus-foratus-helveticus .

# set the startup command to run your binary
CMD ["./caseus-foratus-helveticus"]