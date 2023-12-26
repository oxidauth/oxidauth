FROM --platform=$TARGETPLATFORM debian:12.1

RUN apt-get update -y && apt-get upgrade -y

COPY tmp/oxidauth-server /bin/oxidauth-server

EXPOSE 80

CMD ["/bin/oxidauth-server"]
