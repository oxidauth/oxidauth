FROM --platform=$TARGETPLATFORM debian:12.9

ARG TARGETPLATFORM

COPY tmp/$TARGETPLATFORM/oxidauth-http /bin/oxidauth-http

EXPOSE 80

CMD ["/bin/oxidauth-http"]
