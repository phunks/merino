cross build --target armv7-unknown-linux-musleabi --release

docker compose -f docker-compose_binfmt.yml up -d
cd ..
docker buildx build -f Docker/Dockerfile_musl_armv7 --platform linux/arm/v7 -t merino/armv7 .
docker save merino/armv7:latest > Docker/merino_armv7.tar