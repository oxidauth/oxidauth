FROM --platform=$TARGETPLATFORM debian:12.10

ARG TARGETPLATFORM

COPY tmp/$TARGETPLATFORM/oxidauth-http /bin/oxidauth-http

EXPOSE 80

CMD ["/bin/oxidauth-http"]
