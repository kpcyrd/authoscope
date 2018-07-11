FROM debian:stretch
RUN apt-get update -qq \
    && apt-get install -yq opensmtpd
RUN echo "foo" > /etc/mailname
ADD smtpd.conf /etc/smtpd.conf
EXPOSE 25
CMD ["/usr/sbin/smtpd", "-d"]
