FROM maven:3.8.5-openjdk-18 as mavenBuild
WORKDIR /app
COPY pom.xml .
RUN mvn install -Dmaven.test.skip -B # install will run copy-dependencies
# Copy all other project files and build project
COPY src src
RUN mvn install -Dmaven.test.skip -B


# Now create a minimal JRE using jdeps & jlink
FROM openjdk:18-alpine AS jre-build
WORKDIR /app

COPY --from=mavenBuild /app/target/dependency-jars ./lib
COPY --from=mavenBuild /app/target/*.jar ./app.jar

# find JDK dependencies dynamically from jar
RUN jdeps \
  --ignore-missing-deps \
  -q \
  --multi-release 18 \
  --print-module-deps \
  --class-path /app/lib/* \
  /app/app.jar > ./jre-deps.info

RUN jlink --verbose \
    --compress 2 \
    --strip-java-debug-attributes \
    --no-header-files \
    --no-man-pages \
    --output jre \
    --add-modules $(cat /app/jre-deps.info)


# The proper runnable container, minimal
FROM alpine:latest
WORKDIR /deployment

RUN apk add bash

# copy the custom JRE produced from jlink
COPY --from=jre-build /app/jre jre
# copy the app dependencies
COPY --from=jre-build /app/lib/* .
# copy the app
COPY --from=jre-build /app/app.jar app.jar

# run the app on startup
ENV JAVA_OPTS ""
CMD [ "bash", "-c", "jre/bin/java ${JAVA_OPTS} -jar app.jar" ]
