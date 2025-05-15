FROM alpine:3.12

# Installeer benodigde pakketten
RUN apk add --no-cache \
    bash \
    curl \
    wget \
    coreutils \
    grep \
    sed \
    jq \
    tzdata

# Stel tijdzone in
RUN cp /usr/share/zoneinfo/Europe/Amsterdam /etc/localtime

# Werkmap voor scripts
WORKDIR /scripts

# Standaard commando
CMD ["/bin/bash"]