FROM --platform=$TARGETPLATFORM registry.vizerapp.cloud/lib/debian:12.1-2024.8.0

ARG TARGETPLATFORM

RUN apt-get update && apt-get install --assume-yes libssl-dev libudev-dev

COPY tmp/$TARGETPLATFORM/oxidauth-http /bin/oxidauth-http

EXPOSE 80

CMD ["/bin/oxidauth-http"]
