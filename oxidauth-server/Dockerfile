FROM --platform=$TARGETPLATFORM debian:11.7

RUN apt-get update -y && apt-get upgrade -y && apt-get install openssl

COPY tmp/oxidauth-server /bin/oxidauth-server

EXPOSE 80

CMD ["/bin/oxidauth-server"]
