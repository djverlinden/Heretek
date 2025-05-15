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
    tzdata \
    openssh-server \
    openssh-client

# Stel tijdzone in
RUN cp /usr/share/zoneinfo/Europe/Amsterdam /etc/localtime

# SSH configuratie
RUN mkdir -p /run/sshd && \
    ssh-keygen -A && \
    echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config && \
    echo 'root:alpine' | chpasswd

# Werkmap voor scripts
WORKDIR /scripts

# Expose SSH poort
EXPOSE 22

# Startup script
COPY start.sh /start.sh
RUN chmod +x /start.sh

# Standaard commando
ENTRYPOINT ["/start.sh"]