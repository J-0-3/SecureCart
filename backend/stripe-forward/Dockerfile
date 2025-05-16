FROM stripe/stripe-cli:latest

COPY /listen.sh /bin/listen.sh
RUN chmod +x /bin/listen.sh

ENTRYPOINT ["/bin/sh"]
CMD ["/bin/listen.sh"]
