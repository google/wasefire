FROM ubuntu:latest

# ENV DEBIAN_FRONTEND=noninteractive
COPY scripts/ci-docker.sh /docker.sh
RUN /docker.sh build

USER ubuntu
WORKDIR /home/ubuntu
