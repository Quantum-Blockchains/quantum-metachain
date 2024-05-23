FROM openjdk:17-jdk-alpine

WORKDIR /proxyClient

COPY multiProxy.jar multiProxy.jar
COPY run.sh run.sh

CMD ["/bin/bash"]

ENV JAVAPATH "./run.sh"