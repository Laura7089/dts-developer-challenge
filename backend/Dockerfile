# use backend MSRV (minimum supported rust version)
FROM rust:1.86 AS builder

# retrieve backend application source
COPY . /application
WORKDIR /application

# build the backend with cached artefacts, then move the final product out the cache
RUN \
  --mount=type=cache,target=/application/target \
  --mount=type=cache,target=/usr/local/cargo/registry \
  cargo build --release --locked && \
  mv ./target/release/dts_developer_challenge release_build

# base the container on debian bookworm
FROM debian:bookworm

# copy in the built application and make it executable
COPY --from=builder --chmod=0755 \
  /application/release_build \
  /usr/bin/dts_developer_challenge

# create a low-privilege user:
# - it does not need a home directory
# - it is not an account for a human, hence --system
# - it does not need a login shell, so use `false` to prevent login
RUN useradd \
  -c "Application User" \
  --no-create-home \
  --system \
  --shell /usr/bin/false \
  webuser

# set I/O and parameters
USER webuser
EXPOSE 8080/tcp

# run the service and attach to it
ENTRYPOINT ["/usr/bin/dts_developer_challenge"]
