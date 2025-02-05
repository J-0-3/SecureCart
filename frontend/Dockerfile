FROM nginx
COPY healthcheck.sh /bin/healthcheck.sh
RUN chmod +x /bin/healthcheck.sh
COPY src/ /usr/share/nginx/html
