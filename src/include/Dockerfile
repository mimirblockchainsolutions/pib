FROM mimirbridge/pib-parity:latest

WORKDIR /

COPY ./ ./

WORKDIR /pib

VOLUME /pib/state

RUN chmod +x init.sh && ./init.sh

WORKDIR /

CMD ["--config","pib/config.toml"]

