FROM debian
COPY ./bin/ /root/ciid/bin/
COPY ./src/ /root/ciid/src/
COPY ./build.rs /root/ciid/
COPY ./Cargo* /root/ciid/
WORKDIR /root/ciid
RUN ./bin/install.sh
ENTRYPOINT ["ciid"]
