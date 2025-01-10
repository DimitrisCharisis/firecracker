FROM ubuntu

# Install required packages
RUN apt update && apt install -y init openssh-server sudo

# Create a default user 'myuser' with password 'mypassword'
RUN useradd -m -s /bin/bash myuser && echo 'myuser:mypassword' | chpasswd && \
    usermod -aG sudo myuser

# Enable auto-login for 'myuser' on TTY1
RUN mkdir -p /etc/systemd/system/getty@tty1.service.d && \
    echo "[Service]" > /etc/systemd/system/getty@tty1.service.d/override.conf && \
    echo "ExecStart=" >> /etc/systemd/system/getty@tty1.service.d/override.conf && \
    echo "ExecStart=-/sbin/agetty --autologin myuser --noclear %I \$TERM" >> /etc/systemd/system/getty@tty1.service.d/override.conf
