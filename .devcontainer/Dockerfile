FROM archlinux:latest

RUN pacman -Syu --noconfirm
RUN pacman -S neovim base-devel curl git postgresql-libs redis pgcli \
    ripgrep fd less docker docker-compose sudo --noconfirm
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN bash -c '. ~/.cargo/env && rustup default stable'
RUN bash -c '. ~/.cargo/env && rustup component add rust-analyzer'
RUN bash -c '. ~/.cargo/env && cargo install sqlx-cli'
RUN echo '. ~/.cargo/env' >> ~/.bashrc

WORKDIR /root
CMD ["sleep", "infinity"]
