FROM --platform=$TARGETPLATFORM debian:12.4

RUN apt-get update -y && apt-get upgrade -y

COPY tmp/oxidauth-http /bin/oxidauth-http

EXPOSE 80

CMD ["/bin/oxidauth-http"]
