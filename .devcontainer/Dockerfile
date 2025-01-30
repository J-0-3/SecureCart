FROM archlinux:latest

ARG DB_HOST
ARG DB_PASSWORD
ARG DB_USERNAME
ARG DB_DATABASE
ARG REDIS_HOST

RUN pacman -Syu --noconfirm
RUN pacman -S neovim base-devel curl git postgresql-libs redis pgcli luarocks zellij \
    ripgrep fd less docker docker-compose sed sudo --noconfirm
RUN sed -i 's/\# %wheel/%wheel/' /etc/sudoers
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN bash -c '. ~/.cargo/env && rustup default stable'
RUN bash -c '. ~/.cargo/env && rustup component add rust-analyzer'
RUN bash -c '. ~/.cargo/env && cargo install sqlx-cli'
RUN git clone https://aur.archlinux.org/code-server.git /tmp/code-server
RUN useradd -M -G wheel aur-builder
RUN chown -R aur-builder:aur-builder /tmp/code-server
WORKDIR /tmp/code-server
USER aur-builder
RUN makepkg -si --noconfirm
USER root
RUN userdel -r aur-builder
COPY /config /root/.config
COPY /bashrc /root/.bashrc

WORKDIR /root
ENV DATABASE_URL=postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST/$DB_DATABASE
ENV GIT_DISCOVERY_ACROSS_FILESYSTEM=yes
ENV REDIS_HOST=$REDIS_HOST

CMD ["code-server", "--disable-telemetry", "--disable-update-check", "--auth", "none", "--bind-addr", "0.0.0.0:9999", "app"]
