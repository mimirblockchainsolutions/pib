FROM rust:latest

RUN ["/bin/bash","-c","bash <(curl https://get.parity.io -L | sed 's/sudo //g') -r stable"]

RUN cargo install --git https://github.com/paritytech/parity-ethereum.git ethstore-cli

EXPOSE 8545 8546

ENTRYPOINT ["parity"]
